# og-zkp

> Prove your Bitcoin OG status in zero-knowledge!

og-zkp lets you prove which calendar month you first received Bitcoin, tied to any identity you choose, without exposing your address, transaction, or the specific block it was confirmed in. The output is a compact cryptographic proof that anyone can verify instantly.

You sign a message with a Bitcoin private key. The og-zkp prover will then trace the key back to it's first received transaction, verify that transaction was included in a real Bitcoin block, attests to the calendar month that block was mined and collapses all of that into a single zero-knowledge proof.

The verifier learns only two things: your identity and the month you first received Bitcoin. Nothing else leaks.

## Example

Sign a message with your Bitcoin private key

```
$ bitcoin-cli signmessage 1LukeQU5jwebXbMLDVydeH4vFSobRV9rkj "og-zkp x.com/lukechilds"
HE6QfyPFmJvCGjWohZYAVa+pbdSRjeQpdNbXp6zNbDCnEN65xmK+WYidKlt6J1E/GDJpmLcatjEazVo5wqOg6wM=
```

Run the og-zkp prover

```
$ og-zkp prove --message "og-zkp x.com/lukechilds" --signature "HE6QfyPFmJvCGjWohZYAVa+pbdSRjeQpdNbXp6zNbDCnEN65xmK+WYidKlt6J1E/GDJpmLcatjEazVo5wqOg6wM=" --address 1LukeQU5jwebXbMLDVydeH4vFSobRV9rkj
og-zkp v0.1.0
Looking up first-seen txid for address...
First seen txid: e8e96692d721276a86e8b71ef273df6c45fa44d62f3b73ea0b5d1e3c7f6def4f
Fetching raw tx...
Fetching transaction inclusion proof...
Generating block inclusion proof...
Proving...
Proof generated successfully

OG Status: October 2018
Identity:  x.com/lukechilds

Proof:
og-zkp1r79ssqqqqqqqqq8l6h846jznqy2q0u8trkuxjnvz9xa8g02567kxenkfjed4y4s0a9pcu63g959xk3dwhwyxtcew3z9rqa2jvvfqsw2pc4hlzqaxy29tswd0nd0nsa82uc29g4zyn5a0zqcaaq7vs8nuzv0lmsl0u8qwzu879qp0zsuwwfelfnncep4r4hln0grnrvrtshtl8w6mzujcv4x88mf45kku0ullere7j9xdf2y4zkem3dn87z7nxj9w7cmg0xsjaj
f96g0x25e9pw7wtepukt5wljgklszjmw0d0lt8zadr0xtne8a9yc4ndk2hzn4ccw6k24ad6dh008fhp7jp09z5veveyhgehjpdcdahaqp6ac8gag3r54vn5fqy0y9pkfl5un7ycmxw2j40jassetpjq7vug5f382thh7p348fgdn0c0lxm46xqy5y4w0jvw5hzl402atn4qdaa5vjavwr0tvcv4qn994ujlrgvzxsh5gwxz5jwrz0l96tctpu9frt35uj96g682ae487h
e8nff2d7mz0x560hcvdjxyszjs83qcaffgsaelfx7qnle7h9v4r07h0z69lv7c4vswfvhpjfjctzjw0h3juk2lzwkdt7uxl7qgd6qzvgqxtjcqsptqpm6cyfzpsznjqzvvqq4wnuku5zq8luqex5ppzjgq90sq3xgxm0qqpdd7q63yqrsfqjctew8c8y0e0shqy8yqtxz4dh560r33wgj0unumyxfmanpule7jdxxk9enl8hqn2kndhqcfq56rmef5zttyupwv8apxmea
44zeaf64996rdmhm0meeqvqqq2rww9h
```

Anyone can verify without learning the exact address/block/date

```
$ og-zkp verify og-zkp1r79ssqqqqqqqqq8l6h846jznqy2q0u8trkuxjnvz9xa8g02567kxenkfjed4y4s0a9pcu63g959xk3dwhwyxtcew3z9rqa2jvvfqsw2pc4hlzqaxy29tswd0nd0nsa82uc29g4zyn5a0zqcaaq7vs8nuzv0lmsl0u8qwzu879qp0zsuwwfelfnncep4r4hln0grnrvrtshtl8w6mzujcv4x88mf45kku
0ullere7j9xdf2y4zkem3dn87z7nxj9w7cmg0xsjajf96g0x25e9pw7wtepukt5wljgklszjmw0d0lt8zadr0xtne8a9yc4ndk2hzn4ccw6k24ad6dh008fhp7jp09z5veveyhgehjpdcdahaqp6ac8gag3r54vn5fqy0y9pkfl5un7ycmxw2j40jassetpjq7vug5f382thh7p348fgdn0c0lxm46xqy5y4w0jvw5hzl402atn4qdaa5vjavwr0tvcv4qn994ujlrgvz
xsh5gwxz5jwrz0l96tctpu9frt35uj96g682ae487he8nff2d7mz0x560hcvdjxyszjs83qcaffgsaelfx7qnle7h9v4r07h0z69lv7c4vswfvhpjfjctzjw0h3juk2lzwkdt7uxl7qgd6qzvgqxtjcqsptqpm6cyfzpsznjqzvvqq4wnuku5zq8luqex5ppzjgq90sq3xgxm0qqpdd7q63yqrsfqjctew8c8y0e0shqy8yqtxz4dh560r33wgj0unumyxfmanpule7jd
xxk9enl8hqn2kndhqcfq56rmef5zttyupwv8apxmea44zeaf64996rdmhm0meeqvqqq2rww9h

OG Status: October 2018
Identity:  x.com/lukechilds

✓ Proof is valid
```
