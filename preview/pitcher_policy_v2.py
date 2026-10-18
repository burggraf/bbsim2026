"""Preregistered pure pitching choices; no producer/native/default mode switch."""
from datetime import date
from fractions import Fraction
from preview import season_manager as manager
from preview.usage_ledger import _count

POLICY='season-pool-pitcher-usage-v2'


def _u32(value):
    _count(value)
    if value>2**32-1:raise ValueError('Pitcher count exceeds u32')
    return value


def _clock(completed,scheduled):
    _count(completed);_count(scheduled)
    if not completed<scheduled<=162:raise ValueError('Invalid upcoming pitching clock')
    return completed+1,scheduled


def _wants_long(inning,score_gap):
    _u32(inning)
    if not inning or type(score_gap) is not int:raise ValueError('Invalid relief situation')
    _u32(abs(score_gap))
    return inning<=5 or abs(score_gap)>=4


def relief_rank(usage,target,completed,scheduled,*,pa_sum,outings,inning,score_gap):
    """Exact v2 rank components for a caller-supported empirical/marked-prior outing."""
    g,n=_clock(completed,scheduled)
    _u32(usage);_u32(pa_sum);_u32(outings)
    if not outings:raise ValueError('Unsupported relief workload')
    wants_long=_wants_long(inning,score_gap)
    expected=allowance=None;debt=Fraction(0)
    if target is not None:
        _u32(target)
        expected=Fraction(target*g,n);allowance=max(Fraction(6),expected/5)
        debt=Fraction(5*(target*g-usage*n),max(30*n,target*g))
    preference=max(Fraction(-1),min(Fraction(1),debt))
    long=pa_sum>=6*outings
    return dict(usage=usage,expected=expected,allowance=allowance,debt=debt,
        preference=preference,unknown=target is None,long=long,wants_long=wants_long,
        score=preference+(Fraction(1,4) if long==wants_long else 0))


class Policy(manager.Policy):
    """V2 choices over the unchanged authenticated pool/ledger context.

    Inherited batting/rest stay v1. Caller authenticates role support and workload
    provenance; canonical bench/no-reentry/first-batter legality stays native.
    Decisions never consume usage/RNG or mutate inputs. Budget scales are 80/100.
    """
    def _club_clock(self,tid):
        if tid not in self.pool['teams']:raise ValueError('Unsupported pitching club')
        try:c=self.snapshot['teams'][tid]
        except KeyError as error:raise ValueError('Unsupported pitching club') from error
        return _clock(c['completed_games'],c['scheduled_games'])

    def _usage(self,tid,pid,key):
        return _u32(self.snapshot['players'].get('pitcher:'+pid,{}).get('teams',{}).get(tid,{}).get(key,0))

    def _starter_candidates(self,tid,occupied):
        g,n=self._club_clock(tid);rows=[]
        for pid,p in sorted(self.pool['teams'][tid]['players'].items()):
            starts=_u32(p['starts'])
            if starts>n:raise ValueError('Historical starts exceed club games')
            if not starts or 1 not in p['positions']:continue
            used=self._usage(tid,pid,'starts')
            if used>g-1:raise ValueError('Simulated starts exceed completed club games')
            target=p['targets']['start_outs'];outs=self._usage(tid,pid,'start_outs');scale=100
            if target is not None:
                _u32(target);expected=Fraction(target*g,n);excess=outs-expected
                if excess>36 and excess>Fraction(2,5)*expected:scale=80
            history=self.snapshot['players'].get('pitcher:'+pid,{}).get('workload',[])
            last=max((date.fromisoformat(w['date']) for w in history),default=None)
            excluded='occupied' if pid in occupied else None if self.rested(pid,starter=True) else 'rest'
            rows.append(dict(player_id=pid,starts_target=starts,starts_before=used,
                debt=Fraction(starts*g-used*n,n),last_pitched=last.isoformat() if last else None,
                excluded=excluded,budget_scale=scale))
        return rows

    def starter(self,tid,*,occupied=()):
        rows=self._starter_candidates(tid,set(occupied))
        available=[r for r in rows if r['excluded'] is None]
        if not available:raise ValueError('No rested supported starter')
        chosen=min(available,key=lambda r:(-r['debt'],r['last_pitched'] or '',r['player_id']))
        return dict(player_id=chosen['player_id'],budget_scale=chosen['budget_scale'],
            reason='start_debt',candidates=rows)

    def starters(self,away,home,*,occupied=()):
        """Joint legal identity allocation, scarcer rested club first; no pointer state."""
        if away==home:raise ValueError('Pitching clubs must be distinct')
        teams={'Away':away,'Home':home};busy=set(occupied);choices={}
        order=sorted(teams,key=lambda side:(sum(r['excluded'] is None
            for r in self._starter_candidates(teams[side],busy)),side))
        for side in order:
            choice=self.starter(teams[side],occupied=busy)
            choices[side]=choice;busy.add(choice['player_id'])
        return choices

    def relievers(self,tid,profiles,*,inning,score_gap,occupied=()):
        """Order caller-supported (PA sum,outings) profiles after global rest/identity.

        Profiles must be empirical or explicitly marked priors upstream. Supplying
        a profile is not evidence of source support or a native legal bench entry.
        """
        g,n=self._club_clock(tid);_wants_long(inning,score_gap)
        if not isinstance(profiles,dict):raise ValueError('Invalid relief profiles')
        ranks={};busy=set(occupied)
        for pid,profile in profiles.items():
            p=self._actor(tid,pid)
            if 1 not in p['positions'] or not isinstance(profile,(tuple,list)) or len(profile)!=2:
                raise ValueError('Unsupported relief position/profile')
            ranks[pid]=relief_rank(self._usage(tid,pid,'relief_outs'),p['targets']['relief_outs'],g-1,n,
                pa_sum=profile[0],outings=profile[1],inning=inning,score_gap=score_gap)
        return sorted((pid for pid in ranks if pid not in busy and self.rested(pid,starter=False)),
            key=lambda pid:(-ranks[pid]['score'],-ranks[pid]['debt'],pid))
