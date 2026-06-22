import json

def generate_skeleton():
    joints = [
        "Root",
        "Root/Hull",
        "Root/Hull/Turret",
        "Root/Hull/Turret/Barrel",
        "Root/Hull/Track_L",
        "Root/Hull/Track_R"
    ]

    def identity():
        return "( (1, 0, 0, 0), (0, 1, 0, 0), (0, 0, 1, 0), (0, 0, 0, 1) )"

    usda_content = """#usda 1.0
(
    defaultPrim = "RigRoot"
    metersPerUnit = 1.0
    upAxis = "Z"
)

def SkelRoot "RigRoot"
{
    def Skeleton "Skeleton_Tiger"
    {
        uniform token[] joints = [
"""
    for j in joints:
        usda_content += f'            "{j}",\n'
    usda_content = usda_content[:-2] + "\n        ]\n\n"

    usda_content += "        matrix4d[] bindTransforms = [\n"
    for i in range(len(joints)):
        usda_content += f"            {identity()}{',' if i < len(joints)-1 else ''}\n"
    usda_content += "        ]\n\n"

    usda_content += "        matrix4d[] restTransforms = [\n"
    for i in range(len(joints)):
        usda_content += f"            {identity()}{',' if i < len(joints)-1 else ''}\n"
    usda_content += "        ]\n\n"

    usda_content += """        # Physics schema extensions for limits
        custom float[] physics:jointLimits:lower = [
            0, 0, -180, -10, 0, 0
        ]
        custom float[] physics:jointLimits:upper = [
            0, 0, 180, 20, 0, 0
        ]
        
        # IK Targets definitions (custom schema for pipeline)
        custom dictionary rig:ikTargets = {
            dictionary ik_turret = {
                string joint = "Root/Hull/Turret"
                string targetPrim = "/RigRoot/IK_Targets/Turret_Target"
                string poleVector = ""
            }
            dictionary ik_barrel = {
                string joint = "Root/Hull/Turret/Barrel"
                string targetPrim = "/RigRoot/IK_Targets/Barrel_Target"
                string poleVector = ""
            }
        }

        # Pose Constraints
        custom dictionary rig:poseConstraints = {
            dictionary ground_clamp = {
                string[] constrainedJoints = ["Root/Hull/Track_L", "Root/Hull/Track_R"]
                string constraintType = "PlaneClamp"
                double[] planeNormal = [0, 0, 1]
                double planeDistance = 0.0
            }
        }
    }

    def Xform "IK_Targets"
    {
        def Xform "Turret_Target" {}
        def Xform "Barrel_Target" {}
    }

    # Animation Hook Points
    def Xform "AnimationHookPoints"
    {
        def Xform "WeaponMount_Coaxial"
        {
            custom string skel:attachmentJoint = "Root/Hull/Turret/Barrel"
        }
        def Xform "WeaponMount_Hull"
        {
            custom string skel:attachmentJoint = "Root/Hull"
        }
    }
}
"""
    with open("tiger_tank_skeleton.usda", "w") as f:
        f.write(usda_content)

if __name__ == "__main__":
    generate_skeleton()
