import json

def generate_skeleton():
    joints = [
        "Root",
        "Root/Pelvis",
        "Root/Pelvis/Spine_01",
        "Root/Pelvis/Spine_01/Spine_02",
        "Root/Pelvis/Spine_01/Spine_02/Neck",
        "Root/Pelvis/Spine_01/Spine_02/Neck/Head",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_L",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_L/UpperArm_L",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_L/UpperArm_L/LowerArm_L",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_L/UpperArm_L/LowerArm_L/Hand_L",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_R",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_R/UpperArm_R",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_R/UpperArm_R/LowerArm_R",
        "Root/Pelvis/Spine_01/Spine_02/Shoulder_R/UpperArm_R/LowerArm_R/Hand_R",
        "Root/Pelvis/Thigh_L",
        "Root/Pelvis/Thigh_L/Calf_L",
        "Root/Pelvis/Thigh_L/Calf_L/Foot_L",
        "Root/Pelvis/Thigh_R",
        "Root/Pelvis/Thigh_R/Calf_R",
        "Root/Pelvis/Thigh_R/Calf_R/Foot_R"
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
    def Skeleton "Skeleton_01"
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
            0, 0, -15, -15, -45, -30, 
            -20, -90, -120, -45, 
            -20, -90, -120, -45, 
            -90, -150, -30, 
            -90, -150, -30
        ]
        custom float[] physics:jointLimits:upper = [
            0, 0, 15, 15, 45, 30, 
            20, 90, 0, 45, 
            20, 90, 0, 45, 
            45, 0, 30, 
            45, 0, 30
        ]
        
        # IK Targets definitions (custom schema for pipeline)
        custom dictionary rig:ikTargets = {
            dictionary ik_hand_l = {
                string joint = "Root/Pelvis/Spine_01/Spine_02/Shoulder_L/UpperArm_L/LowerArm_L/Hand_L"
                string targetPrim = "/RigRoot/IK_Targets/Hand_L_Target"
                string poleVector = "/RigRoot/IK_Targets/Elbow_L_Pole"
            }
            dictionary ik_hand_r = {
                string joint = "Root/Pelvis/Spine_01/Spine_02/Shoulder_R/UpperArm_R/LowerArm_R/Hand_R"
                string targetPrim = "/RigRoot/IK_Targets/Hand_R_Target"
                string poleVector = "/RigRoot/IK_Targets/Elbow_R_Pole"
            }
            dictionary ik_foot_l = {
                string joint = "Root/Pelvis/Thigh_L/Calf_L/Foot_L"
                string targetPrim = "/RigRoot/IK_Targets/Foot_L_Target"
                string poleVector = "/RigRoot/IK_Targets/Knee_L_Pole"
            }
            dictionary ik_foot_r = {
                string joint = "Root/Pelvis/Thigh_R/Calf_R/Foot_R"
                string targetPrim = "/RigRoot/IK_Targets/Foot_R_Target"
                string poleVector = "/RigRoot/IK_Targets/Knee_R_Pole"
            }
        }

        # Pose Constraints
        custom dictionary rig:poseConstraints = {
            dictionary ground_clamp = {
                string[] constrainedJoints = ["Root/Pelvis/Thigh_L/Calf_L/Foot_L", "Root/Pelvis/Thigh_R/Calf_R/Foot_R"]
                string constraintType = "PlaneClamp"
                double[] planeNormal = [0, 0, 1]
                double planeDistance = 0.0
            }
            dictionary spine_stiffness = {
                string[] constrainedJoints = ["Root/Pelvis/Spine_01", "Root/Pelvis/Spine_01/Spine_02"]
                string constraintType = "OrientationLimit"
                double stiffness = 0.8
            }
        }
    }

    def Xform "IK_Targets"
    {
        def Xform "Hand_L_Target" {}
        def Xform "Elbow_L_Pole" {}
        def Xform "Hand_R_Target" {}
        def Xform "Elbow_R_Pole" {}
        def Xform "Foot_L_Target" {}
        def Xform "Knee_L_Pole" {}
        def Xform "Foot_R_Target" {}
        def Xform "Knee_R_Pole" {}
    }

    # Animation Hook Points
    def Xform "AnimationHookPoints"
    {
        def Xform "WeaponMount_L"
        {
            custom string skel:attachmentJoint = "Root/Pelvis/Spine_01/Spine_02/Shoulder_L/UpperArm_L/LowerArm_L/Hand_L"
        }
        def Xform "WeaponMount_R"
        {
            custom string skel:attachmentJoint = "Root/Pelvis/Spine_01/Spine_02/Shoulder_R/UpperArm_R/LowerArm_R/Hand_R"
        }
        def Xform "Cockpit_Mount"
        {
            custom string skel:attachmentJoint = "Root/Pelvis/Spine_01"
        }
        def Xform "Jetpack_Mount"
        {
            custom string skel:attachmentJoint = "Root/Pelvis/Spine_01/Spine_02"
        }
    }
}
"""
    with open("skeleton.usda", "w") as f:
        f.write(usda_content)

if __name__ == "__main__":
    generate_skeleton()
