"""Summarize DuckDB HTTPFS EXPLAIN ANALYZE panels; units are display-rounded."""

import argparse
from pathlib import Path
import re


PANEL = re.compile(r'HTTPFS HTTP Stats.*?in:\s*([\d.]+)\s*(bytes|KiB|MiB|GiB).*?#GET:\s*(\d+)', re.S)
SCALE = {'bytes': 1, 'KiB': 1024, 'MiB': 1024**2, 'GiB': 1024**3}


def parse_http_stats(text):
    panels = PANEL.findall(text)
    if not panels:
        raise ValueError('no HTTPFS HTTP Stats panels found')
    return {
        'statements': len(panels),
        'get_requests': sum(int(gets) for _, _, gets in panels),
        'reported_payload_bytes': sum(float(amount) * SCALE[unit] for amount, unit, _ in panels),
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('profile', type=Path)
    args = parser.parse_args()
    print(parse_http_stats(args.profile.read_text()))
