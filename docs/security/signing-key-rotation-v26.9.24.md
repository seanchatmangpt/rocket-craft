# Signing-key rotation (v26.9.24)

Recorded 2026-09-24 (fleet key scan after the single-repo migration). Base `29e52a988290` of `rocket-craft`.
Every private key listed here was committed to this repository and is therefore compromised: every receipt or
attestation signed with it carries no signing authority (standing REFUSED, broken_term R_missing_authority).
The keys leave the tree (history is not rewritten; no force-push), and each key directory's `.gitignore` now
covers both halves. Every checkout keeps its own pair: ggen generates one on first use, and a tracked public
half without its private half would make that first `ggen sync` refuse [FM-KEY-010/011]. The canonical
checkout's new public key is published below for anyone verifying its future receipts.

| key dir | removed private key sha256 | removed public key sha256 | new public key (canonical checkout) |
|---|---|---|---|
| `.agents/challenger_subsystems_m4_1_gen2/.ggen/keys` | `56f1f01665842c743a68f67dd810f03e6fd8af2a2f0b2b2addb453b504179f7d` | `54127f05f64d2e43f5e4ed2892bdcc7e49ff8361475e370519748b7b736e15e0` | `1754c26407a5dc9303fe14586bf8ddb493d478a49736fcc1fe8090f11b0362cd` |
| `.agents/explorer_core_1/temp_validation/.ggen/keys` | `ef9b72db9116b54e087b87c5244fb7f7319ad7291556a131330ebd2ad4113445` | `f47ea853a8dbd1fa0b59205d6bea4613daf392c38da7daa7ab5bcc734e78ffdb` | `2a6b806346b51a3222c134d66eafbae0380dd6ab44fa4504901a3ebb678c5705` |
| `.agents/explorer_reflection_blueprints_3/temp_pack/.ggen/keys` | `7fb479ba04c231da7dfb3047ae36c8b831d801f1a0f22a5b1c59a48d1b0ae0a6` | `b874d5b7ebc54c8b257f809e176c5f8f25e894a7b38fea5b437aa622f7d17ae3` | `b7ad4960ce93b052053ce78fa339c391695983ced524f8b92cd8b5b5f78a6966` |
| `.ggen/keys` | `e688886257a79c6cb497439e3a9c2860c6cec74af031591e1e2a9353e1d9d810` | `c8a67bc431e2b930ace9502db9133ddef3b44c12bbbb8bdcb3255092eae45c1f` | `7047d17ffe46f59cfacf05e9dd5e90f87657fa1586dae45e842591d9db1638e1` |
| `crates/rocket_preue4_verifier/ggen/.ggen/keys` | `7f52285b34d562a900c66708c9cbf0855079fd3abd3cbaca70c02b783c4c945c` | `452a38cdd1653346e427bd58ffa8523df1c98edd2c4cd11161fa66b4fa40748e` | `3010c7226d391db6c68ca2ce2304ac1c2cbf7c6ae430d533b0ac2f71cdc06f7b` |
| `ggen-challenger-verify/.ggen/keys` | `575d6f7dbc0d92eb0ca2e57be1df5d4c6bd89014cb02f6578c3e7dfc09344b0c` | `4dc89bc1867f0f50954805a46d3d635ec8ccfd38c0a4741d50101dee4c85ffc2` | `2ee3361815582741932ed4db8e95057664bbe8a0bcd3561f719aaa9229da1de4` |
| `ggen-test-verify/.ggen/keys` | `a51f346f3a75e53240bdd2dbcdcccb87041258c3158dbd3b3bdc490e0b442097` | `cdbf846ed6f24415c72617a8c84728f9e349c24e6e6201314e8953f6dd03899b` | `d68ccd97bb0837fd83f419a5cda6ca93017e99c1fc699f5f725b0e7e88e8297f` |
| `ggen-validation-tests/.ggen/keys` | `c6c65fa86a1eb33301d6d382c3722de6f2b30c8647f806de72be888f83e42699` | `d8c839427ee0a025a775e3267310116287c8a702ed046b16bcfbfb46f3f760e2` | `a39f76d582c0542becaa5231475ce42305e9df65be67066a68b4cc4d0c58d4d4` |
| `nexus-engine/.ggen/keys` | `37897afe9151842af2ecebbefc626a1e10b688d82cb73135c9ef5e239a476605` | `a1d8b8dfece8f5afcf223294c59d515d9283a98234e1fc8b154dcaa0f3c136b1` | `edc4e456e2d3b06455564cb44ee8313834df4494fe6a5c9ebd31f2a76040933e` |
| `ontology/ggen-packs/mechbirth/.ggen/keys` | `682c2c080edd85c5f5f6da17be534ba430e83c69fed6eb9cb004592b0e11d9c4` | `75b4129b60ccc8ce3a3324e09c57d0f617bc0ce9c98db6f058346c211e1a923b` | `d38818579aa0a41b185a186e93fb6506e40576818793ff834cf9a7241326bcfe` |

## Keys exposed on non-default branches (revoked 2026-09-24)

The v26.9.24 rotation scanned default branches only. A scan of every `origin/*` branch found the
private keys below committed on non-default branches only. Each is compromised and revoked: any receipt
or attestation signed with it carries no signing authority (standing REFUSED, broken_term
R_missing_authority). A disk scan of the canonical checkouts on 2026-09-24 found three of these keys in
use (ggen/packs, ignored files) and replaced them with fresh pairs. Copies in agent worktrees and tool
caches may still hold them. History is not rewritten, so the branches keep the blobs.

| path | private key sha256 | derived public key | branches (count, first) |
|---|---|---|---|
| `ontology/.ggen/keys/signing.key` | `f0285cc0fca702e5382457801830bd541f859f015dbd83f0101dff195406dd4e` | `f8041d15cc1f0423fba2b7072c32befe6b94be944d4f9198bf5b6a31bd6ba3b0` | 1, `chore/remove-ds-store` |
| `.ggen/keys/signing.key` | `7535bc95ad5f4d3d1c41d45abdb5cfe3416144323b6b752aaf29a2998e81ab00` | `458b0eb7cbcedcf29e2eef9fce8fcd5e75a1511f3b0c7f5abce215b9a1eaec2c` | 7, `claude/beautiful-bardeen-yubn42` |
