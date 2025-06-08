use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub fn activate_effect() -> EffectAsset {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 1., 0., 0.7));
    gradient.add_key(1.0, Vec4::new(1., 1., 0.5, 0.0));

    // Create a new expression module
    let mut module = Module::default();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.01),
        dimension: ShapeDimension::Surface,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(2.),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = module.lit(0.7); // literal value "10.0"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., -2., 0.));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    EffectAsset::new(
        // Maximum number of particles alive at a time
        600,
        // Spawn at a rate of 5 particles per second
        SpawnerSettings::rate(600.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("MyEffect")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .update(update_accel)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant(Vec3::ONE * 0.05),
        screen_space_size: false,
    })
    .render(ColorOverLifetimeModifier {
        gradient,
        ..default()
    })
}

pub fn delete_effect() -> EffectAsset {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 0.2, 0., 0.7));
    gradient.add_key(1.0, Vec4::new(1., 0.2, 0.5, 0.0));

    // Create a new expression module
    let mut module = Module::default();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.01),
        dimension: ShapeDimension::Surface,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(2.),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = module.lit(0.7); // literal value "10.0"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., -2., 0.));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    EffectAsset::new(
        // Maximum number of particles alive at a time
        600,
        // Spawn at a rate of 5 particles per second
        SpawnerSettings::rate(600.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("MyEffect")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .update(update_accel)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant(Vec3::ONE * 0.05),
        screen_space_size: false,
    })
    .render(ColorOverLifetimeModifier {
        gradient,
        ..default()
    })
}
