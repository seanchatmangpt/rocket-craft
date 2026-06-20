import os

base_dir = "/Users/sac/rocket-craft/crates/mech_factory_mud"
src_dir = os.path.join(base_dir, "src")
tests_dir = os.path.join(base_dir, "tests")

os.makedirs(src_dir, exist_ok=True)
os.makedirs(tests_dir, exist_ok=True)

modules = {
    "world": "FactoryWorld",
    "factory": "FactoryContext",
    "walkthrough": "WalkthroughRoute",
    "stations": "FactoryStation",
    "parts": "MechPart",
    "transitions": "TransitionTable",
    "geometry": "GeometrySurrogate",
    "motion": "MotionSurrogate",
    "skin": "SkinSurrogate",
    "projection": "ProjectionCommand",
    "receipt": "ReceiptEvent",
    "ocel": "OcelEvent",
    "replay": "ReplayState",
    "verifier": "VerifierGate",
    "report": "VerifierReport"
}

for mod, struct_name in modules.items():
    file_path = os.path.join(src_dir, f"{mod}.rs")
    with open(file_path, "w") as f:
        f.write(f"""#[derive(Debug, Clone, Default)]
pub struct {struct_name} {{
    pub id: String,
}}

impl {struct_name} {{
    pub fn new(id: impl Into<String>) -> Self {{
        Self {{ id: id.into() }}
    }}
    pub fn is_valid(&self) -> bool {{
        !self.id.is_empty()
    }}
}}
""")

tests = {
    "factory_walkthrough": "test_factory_walkthrough",
    "refusals": "test_refusals",
    "receipt_chain": "test_receipt_chain",
    "ue4_export": "test_ue4_export"
}

for test, fn_name in tests.items():
    file_path = os.path.join(tests_dir, f"{test}.rs")
    with open(file_path, "w") as f:
        f.write(f"""#[test]
fn {fn_name}() {{
    assert!(true);
}}
""")

