//! Demo 17: Resource Management
//!
//! This example demonstrates the resource management system:
//! - ResourceManager for organizing assets
//! - ResourcePackage for grouping related resources
//! - ResourceEntry for individual assets with metadata
//! - Resource type detection from file extensions
//! - Search across packages
//! - Path resolution
//!
//! Run with: `cargo run --example 17_resource_management`

use flux_graph::resource::{ResourceEntry, ResourceManager, ResourcePackage, ResourceType};
use std::path::PathBuf;

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║ Demo 17: Resource Management           ║");
    println!("╚════════════════════════════════════════╝\n");

    // Create a resource manager
    let mut manager = ResourceManager::new();
    println!("Created ResourceManager");

    // Create a core/built-in package (read-only)
    let mut core_pkg = ResourcePackage::read_only("Core", PathBuf::from("/app/operators"));
    core_pkg.description = Some("Built-in operators and symbols".to_string());
    core_pkg.version = Some("1.0.0".to_string());

    println!("\nCore package (read-only):");
    println!("  Name: {}", core_pkg.name);
    println!("  Path: {:?}", core_pkg.path);
    println!("  Read-only: {}", core_pkg.is_read_only);

    // Create a user project package
    let mut project_pkg = ResourcePackage::new("MyProject", PathBuf::from("/projects/demo"));
    project_pkg.description = Some("User project with custom assets".to_string());

    // Add symbol resources (using .flux extension now)
    project_pkg.add_resource(ResourceEntry::from_path("symbols/MainScene.flux"));
    project_pkg.add_resource(ResourceEntry::from_path("symbols/Effects/Bloom.flux"));
    project_pkg.add_resource(ResourceEntry::from_path("symbols/Effects/ColorGrade.flux"));

    // Add audio resources with metadata
    let audio1 = ResourceEntry::new("BackgroundMusic", "audio/music/ambient.mp3")
        .with_metadata("artist", "Composer Name")
        .with_metadata("duration", "240")
        .with_size(5 * 1024 * 1024);
    project_pkg.add_resource(audio1);

    let audio2 = ResourceEntry::from_path("audio/sfx/click.wav").with_size(50 * 1024);
    project_pkg.add_resource(audio2);

    // Add texture resources
    project_pkg.add_resource(ResourceEntry::from_path("textures/player_diffuse.png"));
    project_pkg.add_resource(ResourceEntry::from_path("textures/player_normal.png"));
    project_pkg.add_resource(ResourceEntry::from_path("textures/environment_hdr.jpg"));

    // Add shader resources
    project_pkg.add_resource(ResourceEntry::from_path("shaders/custom_effect.hlsl"));

    println!("\nProject package:");
    println!("  Name: {}", project_pkg.name);
    println!("  Total resources: {}", project_pkg.resource_count());

    // Display resources by type
    println!("\n  Symbols:");
    for entry in project_pkg.resources_of_type(ResourceType::Symbol) {
        println!("    - {}", entry.relative_path);
    }

    println!("\n  Audio:");
    for entry in project_pkg.resources_of_type(ResourceType::Audio) {
        let size = entry
            .size_bytes
            .map(|s| format!("{}KB", s / 1024))
            .unwrap_or("?".to_string());
        println!("    - {} [{}]", entry.relative_path, size);
        if !entry.metadata.is_empty() {
            for (k, v) in &entry.metadata {
                println!("      {}: {}", k, v);
            }
        }
    }

    println!("\n  Images:");
    for entry in project_pkg.resources_of_type(ResourceType::Image) {
        println!("    - {}", entry.relative_path);
    }

    println!("\n  Shaders:");
    for entry in project_pkg.resources_of_type(ResourceType::Shader) {
        println!("    - {}", entry.relative_path);
    }

    // Add packages to manager
    let _core_id = manager.add_package(core_pkg);
    let project_id = manager.add_package(project_pkg);

    println!("\nResource Manager status:");
    println!("  Packages: {}", manager.package_count());
    println!("  Total resources: {}", manager.total_resource_count());

    // List all packages
    println!("\n  Package list:");
    for name in manager.package_names() {
        let pkg = manager.get_package_by_name(name).unwrap();
        println!(
            "    - {} ({} resources, read-only: {})",
            name,
            pkg.resource_count(),
            pkg.is_read_only
        );
    }

    // Find resources across packages
    println!("\nSearching for resources:");

    if let Some((pkg, entry)) = manager.find_resource("audio/music/ambient.mp3") {
        println!("  Found 'ambient.mp3' in package '{}'", pkg.name);
        println!("    Type: {:?}", entry.resource_type);
        println!(
            "    Full path: {:?}",
            pkg.resolve_path(&entry.relative_path)
        );
    }

    // Find all audio resources
    let all_audio = manager.find_resources_of_type(ResourceType::Audio);
    println!(
        "\n  All audio resources across packages: {}",
        all_audio.len()
    );
    for (pkg, entry) in &all_audio {
        println!("    [{}/{}] {}", pkg.name, entry.name, entry.relative_path);
    }

    // Demonstrate resource type detection
    println!("\nResource type detection:");
    let extensions = [
        "flux", "wav", "mp3", "png", "jpg", "mp4", "ttf", "hlsl", "json", "xyz",
    ];
    for ext in extensions {
        let rtype = ResourceType::from_extension(ext);
        println!("  .{} -> {:?}", ext, rtype);
    }

    // Path resolution
    println!("\nPath resolution examples:");
    if let Some(pkg) = manager.get_package(project_id) {
        println!(
            "  textures/player_diffuse.png -> {:?}",
            pkg.resolve_path("textures/player_diffuse.png")
        );
    }

    // Add search paths
    manager.add_search_path(PathBuf::from("/shared/assets"));
    manager.add_search_path(PathBuf::from("/library/common"));
    println!("\n  Added 2 search paths for fallback resolution");
}
