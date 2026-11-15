use pack::{
    Channel, ChannelKind, Clip, ConvexCollider, Interpolation, Joint, Marker, Model, ModelPack,
    PackVertex, Skeleton, PACK_VERSION,
};

#[test]
fn test_pack_vertex_size_and_alignment() {
    assert_eq!(std::mem::size_of::<PackVertex>(), 32);
    assert_eq!(std::mem::align_of::<PackVertex>(), 4);
    assert_eq!(std::mem::offset_of!(PackVertex, position), 0);
    assert_eq!(std::mem::offset_of!(PackVertex, normal), 12);
    assert_eq!(std::mem::offset_of!(PackVertex, color), 24);
    assert_eq!(std::mem::offset_of!(PackVertex, bone), 28);
    assert_eq!(std::mem::offset_of!(PackVertex, _pad), 29);
}

#[test]
fn test_pack_roundtrip() {
    let temp_dir = std::env::temp_dir().join("rotland_pack_test");
    let pack_path = temp_dir.join("test.pack");

    let vertex = PackVertex {
        position: [1.0, 2.0, 3.0],
        normal: [0.0, 1.0, 0.0],
        color: [74, 59, 46, 255],
        bone: 3,
        _pad: [0; 3],
    };

    let marker = Marker {
        name: "loot_1".to_string(),
        translation: [1.0, 0.0, -2.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    };

    let collider = ConvexCollider {
        name: "box-convcolonly".to_string(),
        points: vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
    };

    let joint = Joint {
        name: "root".to_string(),
        parent: None,
        inverse_bind: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
        rest_translation: [0.0, 0.5, 0.0],
        rest_rotation: [0.0, 0.0, 0.0, 1.0],
        rest_scale: [1.0, 1.0, 1.0],
    };

    let channel = Channel {
        joint: 0,
        kind: ChannelKind::Translation,
        interpolation: Interpolation::Linear,
        times: vec![0.0, 0.5, 1.0],
        values: vec![[0.0, 0.0, 0.0, 0.0], [0.0, 0.2, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0]],
    };

    let clip = Clip {
        name: "idle".to_string(),
        duration: 1.0,
        channels: vec![channel],
    };

    let model = Model {
        name: "test_model".to_string(),
        folder: "props".to_string(),
        vertices: vec![vertex],
        bounds_min: [1.0, 2.0, 3.0],
        bounds_max: [1.0, 2.0, 3.0],
        markers: vec![marker],
        colliders: vec![collider],
        skeleton: Some(Skeleton {
            joints: vec![joint],
        }),
        clips: vec![clip],
    };

    let pack = ModelPack {
        version: PACK_VERSION,
        models: vec![model.clone()],
    };

    pack.save(&pack_path).expect("failed to save pack");
    let loaded = ModelPack::load(&pack_path).expect("failed to load pack");

    assert_eq!(pack, loaded);
    assert_eq!(loaded.find("test_model"), Some(&model));
    assert_eq!(loaded.find("non_existent"), None);

    let _ = std::fs::remove_file(pack_path);
    let _ = std::fs::remove_dir(temp_dir);
}
