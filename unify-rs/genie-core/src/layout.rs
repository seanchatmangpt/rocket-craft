use crate::spec::{WorldSpec, Place, Actor, Object, Vector3};

/// Compiles a validated `WorldSpec` into a copy-pasteable Unreal Engine 4 compatible `.t3d` level map string.
pub struct LayoutCompiler;

impl LayoutCompiler {
    /// Compiles the validated WorldSpec into a copy-pasteable UE4 T3D map string
    pub fn compile(spec: &WorldSpec) -> String {
        let mut out = String::new();

        out.push_str("Begin Map\n   Begin Level\n");

        // Map place IDs to their center positions for relative positioning of actors/objects
        let mut place_centers = std::collections::HashMap::new();
        for place in &spec.places {
            place_centers.insert(place.id.clone(), place.bounds.center);
            out.push_str(&Self::compile_place(place));
        }

        for actor in &spec.actors {
            let parent_center = place_centers.get(&actor.place_id).copied().unwrap_or_default();
            out.push_str(&Self::compile_actor(actor, parent_center));
        }

        for object in &spec.objects {
            let parent_center = place_centers.get(&object.place_id).copied().unwrap_or_default();
            out.push_str(&Self::compile_object(object, parent_center));
        }

        out.push_str("   End Level\nEnd Map\n");

        out
    }

    fn compile_place(place: &Place) -> String {
        let center = place.bounds.center;
        let half_extents = place.bounds.half_extents;
        let x = center.x;
        let y = center.y;
        let z = center.z - half_extents.z - 50.0;
        let scale_x = half_extents.x / 50.0;
        let scale_y = half_extents.y / 50.0;
        let scale_z = 1.0;
        let actor_name = format!("Place_{}", place.id);
        let actor_label = format!("Floor_{}", place.name);

        format!(
            "      Begin Actor Class=StaticMeshActor Name={actor_name} Archetype=StaticMeshActor'/Script/Engine.Default__StaticMeshActor'\n\
            \x20        Begin Object Class=StaticMeshComponent Name=StaticMeshComponent0 ObjName=StaticMeshComponent0 Archetype=StaticMeshComponent'/Script/Engine.Default__StaticMeshActor:StaticMeshComponent0'\n\
            \x20        End Object\n\
            \x20        Begin Object Name=\"StaticMeshComponent0\"\n\
            \x20           StaticMesh=StaticMesh'/Engine/BasicShapes/Cube.Cube'\n\
            \x20           RelativeLocation=(X={x:.6},Y={y:.6},Z={z:.6})\n\
            \x20           RelativeRotation=(Pitch=0.000000,Yaw=0.000000,Roll=0.000000)\n\
            \x20           RelativeScale3D=(X={sx:.6},Y={sy:.6},Z={sz:.6})\n\
            \x20        End Object\n\
            \x20        StaticMeshComponent=StaticMeshComponent0\n\
            \x20        RootComponent=StaticMeshComponent0\n\
            \x20        ActorLabel=\"{label}\"\n\
            \x20     End Actor\n",
            actor_name = actor_name,
            x = x,
            y = y,
            z = z,
            sx = scale_x,
            sy = scale_y,
            sz = scale_z,
            label = actor_label
        )
    }

    fn get_class_info(role_or_class: &str) -> (String, String) {
        let class_path = if role_or_class.starts_with('/') {
            role_or_class.to_string()
        } else {
            format!("/Game/BP_{}.BP_{}_C", role_or_class, role_or_class)
        };
        let class_name = if let Some(dot_idx) = class_path.rfind('.') {
            class_path[dot_idx + 1..].to_string()
        } else if let Some(slash_idx) = class_path.rfind('/') {
            class_path[slash_idx + 1..].to_string()
        } else {
            class_path.clone()
        };
        (class_path, class_name)
    }

    fn compile_actor(actor: &Actor, parent_center: Vector3) -> String {
        let rel_pos = actor.placement.position;
        let rot = actor.placement.rotation;
        let abs_x = parent_center.x + rel_pos.x;
        let abs_y = parent_center.y + rel_pos.y;
        let abs_z = parent_center.z + rel_pos.z;

        let (class_path, class_name) = Self::get_class_info(&actor.role);
        let actor_name = format!("Actor_{}", actor.id);
        
        let archetype_path = if let Some(dot_idx) = class_path.rfind('.') {
            format!("{}.Default__{}", &class_path[..dot_idx], &class_path[dot_idx + 1..])
        } else {
            class_path.clone()
        };

        format!(
            "      Begin Actor Class={class_path} Name={actor_name} Archetype={class_path}'{archetype_path}'\n\
            \x20        Begin Object Class=StaticMeshComponent Name=StaticMeshComponent0 ObjName=StaticMeshComponent0\n\
            \x20        End Object\n\
            \x20        Begin Object Name=\"StaticMeshComponent0\"\n\
            \x20           StaticMesh=StaticMesh'/Engine/BasicShapes/Cylinder.Cylinder'\n\
            \x20           RelativeLocation=(X={x:.6},Y={y:.6},Z={z:.6})\n\
            \x20           RelativeRotation=(Pitch={pitch:.6},Yaw={yaw:.6},Roll={roll:.6})\n\
            \x20           RelativeScale3D=(X=1.000000,Y=1.000000,Z=2.000000)\n\
            \x20        End Object\n\
            \x20        StaticMeshComponent=StaticMeshComponent0\n\
            \x20        RootComponent=StaticMeshComponent0\n\
            \x20        ActorLabel=\"{label}\"\n\
            \x20     End Actor\n",
            class_path = class_path,
            actor_name = actor_name,
            archetype_path = archetype_path,
            x = abs_x,
            y = abs_y,
            z = abs_z,
            pitch = rot.x,
            yaw = rot.y,
            roll = rot.z,
            label = actor.name
        )
    }

    fn compile_object(object: &Object, parent_center: Vector3) -> String {
        let rel_pos = object.placement.position;
        let rot = object.placement.rotation;
        let abs_x = parent_center.x + rel_pos.x;
        let abs_y = parent_center.y + rel_pos.y;
        let abs_z = parent_center.z + rel_pos.z;

        let (class_path, class_name) = Self::get_class_info(&object.class);
        let actor_name = format!("Object_{}", object.id);

        let archetype_path = if let Some(dot_idx) = class_path.rfind('.') {
            format!("{}.Default__{}", &class_path[..dot_idx], &class_path[dot_idx + 1..])
        } else {
            class_path.clone()
        };

        format!(
            "      Begin Actor Class={class_path} Name={actor_name} Archetype={class_path}'{archetype_path}'\n\
            \x20        Begin Object Class=StaticMeshComponent Name=StaticMeshComponent0 ObjName=StaticMeshComponent0\n\
            \x20        End Object\n\
            \x20        Begin Object Name=\"StaticMeshComponent0\"\n\
            \x20           StaticMesh=StaticMesh'/Engine/BasicShapes/Sphere.Sphere'\n\
            \x20           RelativeLocation=(X={x:.6},Y={y:.6},Z={z:.6})\n\
            \x20           RelativeRotation=(Pitch={pitch:.6},Yaw={yaw:.6},Roll={roll:.6})\n\
            \x20           RelativeScale3D=(X=1.000000,Y=1.000000,Z=1.000000)\n\
            \x20        End Object\n\
            \x20        StaticMeshComponent=StaticMeshComponent0\n\
            \x20        RootComponent=StaticMeshComponent0\n\
            \x20        ActorLabel=\"{label}\"\n\
            \x20     End Actor\n",
            class_path = class_path,
            actor_name = actor_name,
            archetype_path = archetype_path,
            x = abs_x,
            y = abs_y,
            z = abs_z,
            pitch = rot.x,
            yaw = rot.y,
            roll = rot.z,
            label = object.name
        )
    }
}
