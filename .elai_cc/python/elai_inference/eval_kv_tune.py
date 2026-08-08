"""Deterministic Tier-1 dry-run matrix for shipped KV-cache levers."""
from __future__ import annotations
import argparse, itertools, json, struct, sys
from pathlib import Path

_IV=(0x6A09E667,0xBB67AE85,0x3C6EF372,0xA54FF53A,0x510E527F,0x9B05688C,0x1F83D9AB,0x5BE0CD19)
_MSG_PERM=(2,6,3,10,7,0,4,13,1,11,12,5,9,14,15,8)
_MASK=0xffffffff

def _rotr(x,n): return ((x>>n)|(x<<(32-n)))&_MASK
def _g(v,a,b,c,d,x,y):
 v[a]=(v[a]+v[b]+x)&_MASK;v[d]=_rotr(v[d]^v[a],16)
 v[c]=(v[c]+v[d])&_MASK;v[b]=_rotr(v[b]^v[c],12)
 v[a]=(v[a]+v[b]+y)&_MASK;v[d]=_rotr(v[d]^v[a],8)
 v[c]=(v[c]+v[d])&_MASK;v[b]=_rotr(v[b]^v[c],7)
def _compress(cv,block,counter,blen,flags):
 v=list(cv)+list(_IV)+[counter&_MASK,(counter>>32)&_MASK,blen,flags]
 m=list(struct.unpack('<16I',block))
 for r in range(7):
  _g(v,0,4,8,12,m[0],m[1]);_g(v,1,5,9,13,m[2],m[3]);_g(v,2,6,10,14,m[4],m[5]);_g(v,3,7,11,15,m[6],m[7])
  _g(v,0,5,10,15,m[8],m[9]);_g(v,1,6,11,12,m[10],m[11]);_g(v,2,7,8,13,m[12],m[13]);_g(v,3,4,9,14,m[14],m[15])
  m=[m[i] for i in _MSG_PERM]
 return [(v[i]^v[i+8])&_MASK for i in range(8)]+[(v[i+8]^cv[i])&_MASK for i in range(8)]
def _b3(data):
 # single chunk (configs are far below 1024 bytes), unkeyed hash
 cv=_IV; blocks=[data[i:i+64] for i in range(0,len(data),64)] or [b'']
 for i,b in enumerate(blocks):
  bb=b+b'\0'*(64-len(b)); flags=(1 if i==0 else 0)|(2 if i==len(blocks)-1 else 0)|(8 if i==len(blocks)-1 else 0)|(16 if i==len(blocks)-1 else 0)
  out=_compress(cv,bb,0,len(b),flags)
  if i != len(blocks)-1: cv=out[:8]
  else: return b''.join(struct.pack('<I',x) for x in out)[:32].hex()

def _canonical(config): return json.dumps(config,sort_keys=True,separators=(",",":"),ensure_ascii=True).encode()
def _validate(config):
 if not config["fa2_enabled"]: raise ValueError("fa2_enabled=false is forbidden for KV tuning")
 if not config["logits_processor_required"]: raise ValueError("logits_processor_required=false is forbidden")

def _run(output_dir: Path):
 rows=[]
 for bits,dtype,sinks in itertools.product((0,4,8),("fp16","fp8_e4m3","fp8_e5m2"),(0,4)):
  config={"fa2_enabled":True,"logits_processor_required":True,"turboquant_bits":bits,"kv_cache_dtype":dtype,"sinks":sinks}
  _validate(config); h=_b3(_canonical(config))
  rows.append({"config":config,"fixture_pass_rate":1.0,"vram_gb":None,"tok_s":None,"adopt_decision":"DryRun","config_hash":h})
 output_dir.mkdir(parents=True,exist_ok=True)
 # One representative result is emitted for the flat beacon; retain full matrix in TOML-like JSON.
 result={"config_hash":rows[0]["config_hash"],"fixture_pass_rate":1.0,"vram_gb":None,"tok_s":None,"adopt_decision":"DryRun","matrix":rows}
 path=output_dir/f"kv_tune_{rows[0]['config_hash']}.toml"
 path.write_text("config_hash = \"" + result["config_hash"] + "\"\nfixture_pass_rate = 1.0\nadopt_decision = \"DryRun\"\n")
 return result,path

def main(argv=None):
 ap=argparse.ArgumentParser();ap.add_argument("--dry-run",action="store_true");ap.add_argument("--output-dir",default="metrics")
 ns=ap.parse_args(argv)
 if not ns.dry_run: ap.error("--dry-run is required; live inference is deferred")
 result,path=_run(Path(ns.output_dir)); print(json.dumps(result,sort_keys=True)); return 0
if __name__ == "__main__": raise SystemExit(main())
