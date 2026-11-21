#!/usr/bin/env python3
"""Add read-only diagnostics to the ephemeral pinned FlintCLI checkout."""
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
needle = '            client.start_use_item();'
assert text.count(needle) == 1
trace = '''            let probe_pose = format!("{:?}", *client.component::<azalea::entity::Pose>()?);
            let probe_dimensions = format!("{:?}", *client.component::<azalea::entity::dimensions::EntityDimensions>()?);
            let probe_hit = format!("{:?}", *client.component::<azalea::interact::pick::HitResultComponent>()?);
            println!("FLINT_PROBE position={:?} pose={} dimensions={} hit={}", client.position()?, probe_pose, probe_dimensions, probe_hit);
'''
path.write_text(text.replace(needle, trace + needle))
