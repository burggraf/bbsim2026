"""Pinned offline witness export for the closed-starting-roster experiment."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile
from audits.raw_mechanisms import unique_object

REPORT_SHA='5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b'
SNAPSHOT_SHA='d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528'
SUPPLEMENT_SHA='554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82'
ATTRIBUTION='The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def literal(value):
    if not isinstance(value,str) or not value.strip() or any(ord(c)<32 or 127<=ord(c)<=159 for c in value):
        raise ValueError('Invalid literal lineup field')
    return value


def encode_witness(witness):
    if witness['use_dh'] is not True or (witness['game_id'],witness['date'],witness['park_id']) != ('OAK201903200','2019-03-20','TOK01'):
        raise ValueError('Unsupported starting witness')
    teams=witness['teams']
    if len(teams)!=2 or {t['side'] for t in teams}!={'Away','Home'}:
        raise ValueError('Invalid starting sides')
    teams=sorted(teams,key=lambda t:0 if t['side']=='Away' else 1)
    metadata=dict(format='closed-starting-lineup-v1',game_id=witness['game_id'],date=witness['date'],park_id=witness['park_id'],
        away_team=teams[0]['team_name'],home_team=teams[1]['team_name'],audit_sha256=REPORT_SHA,
        snapshot_sha256=SNAPSHOT_SHA,supplement_sha256=SUPPLEMENT_SHA,attribution=ATTRIBUTION)
    rows=[];all_ids=set()
    for t,team_id in zip(teams,['SEA','OAK']):
        if t['team_id']!=team_id:raise ValueError('Unsupported starting team')
        order=t['batting_order'];field=t['fielding']
        if (len(order)!=9 or any(type(p['slot']) is not int for p in order)
                or {p['slot'] for p in order}!=set(range(1,10))
                or len(field)!=10 or any(type(p['position']) is not int for p in field)
                or {p['position'] for p in field}!=set(range(1,11))
                or type(t['dh_slot']) is not int or type(t['engine_dh_slot']) is not int):
            raise ValueError('Invalid starting slot/position grain')
        by_id={p['player_id']:p for p in field}
        if (len(by_id)!=10 or len({p['player_id'] for p in order})!=9
                or {p['player_id'] for p in order}!={p['player_id'] for p in field if p['position']!=1}
                or next(p['player_id'] for p in field if p['position']==1)!=t['pitcher_id']
                or t['engine_dh_slot']!=t['dh_slot']-1
                or next(p['slot'] for p in order if by_id[p['player_id']]['position']==10)!=t['dh_slot']):
            raise ValueError('Invalid pitcher/DH/defense relationship')
        pitcher=by_id[t['pitcher_id']]
        for slot,p in [(0,pitcher)]+[(p['slot'],p) for p in sorted(order,key=lambda p:p['slot'])]:
            player=literal(p['player_id']);name=literal(p['name'])
            if not re.fullmatch('[a-z0-9]+',player) or player in all_ids or name!=by_id[player]['name']:
                raise ValueError('Invalid starting identity/name relationship')
            all_ids.add(player)
            rows.append((f"{t['side']}:{slot}:{by_id[player]['position']}:{player}",name))
    return 'key\tvalue\n'+''.join(literal(k)+'\t'+literal(v)+'\n' for k,v in [*metadata.items(),*rows])


def publish(text,source,pin,output):
    temporary=None
    try:
        with tempfile.NamedTemporaryFile(mode='w',encoding='utf-8',newline='',dir=output.parent,delete=False) as stream:
            temporary=Path(stream.name)
            stream.write(text);stream.flush();os.fsync(stream.fileno())
        if digest(source)!=pin:
            raise ValueError('Lineup input changed before publication')
        os.link(temporary,output)  # Atomic, exclusive publication of complete bytes.
    finally:
        if temporary is not None:temporary.unlink()


def export(report,output):
    if output.exists():raise ValueError('Refusing to overwrite lineup output')
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError('Lineup output must be outside repository')
    if digest(report)!=REPORT_SHA:raise ValueError('Unsupported lineup report digest')
    data=json.loads(report.read_text(),object_pairs_hook=unique_object)
    provenance=data['provenance']
    if (data['format']!='2019-lineup-readiness-v1' or provenance['snapshot_sha256']!=SNAPSHOT_SHA
            or provenance['supplement_sha256']!=SUPPLEMENT_SHA or provenance['attribution']!=ATTRIBUTION):
        raise ValueError('Unsupported lineup report provenance')
    text=encode_witness(data['witnesses']['primary'])
    publish(text,report,REPORT_SHA,output)
    return text


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();text=export(args.report,args.output)
    print(f'Exported closed starting lineup: {len(text.encode())} bytes; SHA256 {digest(args.output)}')


if __name__=='__main__':main()
