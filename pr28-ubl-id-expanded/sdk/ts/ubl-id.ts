// sdk/ts/ubl-id.ts — helpers para Pessoas, LLM e Apps
// npm i @noble/ed25519
import * as ed from '@noble/ed25519';

export type CreateAgentInput = {
  kind: 'llm'|'app';
  display_name: string;
  public_key?: string; // hex; se ausente, gera
};

export async function createAgent(base: string, input: CreateAgentInput){
  let priv: string|undefined;
  let pub = input.public_key;
  if(!pub){
    const sk = ed.utils.randomPrivateKey();
    const pk = await ed.getPublicKeyAsync(sk);
    priv = Buffer.from(sk).toString('hex');
    pub  = Buffer.from(pk).toString('hex');
  }
  const res = await fetch(`${base}/id/agents`, {
    method:'POST', headers:{'Content-Type':'application/json'},
    body: JSON.stringify({ kind: input.kind, display_name: input.display_name, public_key: pub })
  });
  if(!res.ok) throw new Error(`createAgent failed: ${res.status}`);
  const json = await res.json();
  return { ...json, private_key: priv };
}
