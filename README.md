# Bevy Systems

A collection of bevy examples that can be used for creating a game.

### Quickstart

Check examples.

### Tasks

- [x] async hexmap generation - Implement async hexmap generation in lib.rs

### Examples

- [x] hexmap_overlay.rs - Visualize terrain data overlays (e.g. height, temperature, noise values) on a hexagonal grid.
    - Color tiles by their noise value (height: grayscale, temperature: blue to red gradient).
    - Switch between different overlays (height, temperature, etc.) using `UP` and `DOWN` keys.
- [x] hexmap_coordinates.rs - Display the coordinates of a hex in the tile grid.
    - Show coordinates on top of each hex tile.
    - Switch between different coordinate systems (e.g. axial, offset) using `UP` and `DOWN` keys.
- [x] biome_classification.rs - Classify hex tiles into biomes based on noise values.
    - Use a simple classification algorithm to assign biomes (e.g. grassland, desert, forest) based on height and temperature.
    - Display hex tiles with different colors based on their biome (e.g yellow for desert, green for grassland, dark green for forest).
- [x] hexmap_resources.rs - Generate resources on hex tiles based on noise values.
    - Compute density of resources based on the noise values (temperature, height, moisture, etc.)
    - Use hexmap information to generate resource density maps (e.g. tree, ore, food).
    - Display resources on hex tiles with colors representing resource density (e.g. green for trees, gray for ore, yellow for food).
    - Switch between different resource types using `UP` and `DOWN` keys.
    - Add a threshold metter to filter out low-density resources and display only high-density resources.
    - Maybe add a noisemap for resources, to add more randomness to the resource distribution.
- [ ] generic noise function
    + implement a noise function that can return something else than a f64 => resource type and density (the f64 confidence level)
    + implement all noise values to be between 0.0 and 1.0, so we can use them as a confidence level
    + refactor the render plugin to use TileKind instead of computing it
    + implement the feature planet function and add some tree meshes
    - use the resource type with the tile kind and confidence to tell if I should place that thing in the tile: how can I add variants? (oak tree in plains, spruce in hills etc)
    - we can have a "variants" map in the resource which has variants based on tile kind (I would rather use maps for simplicity, but we can try to use Vec too)
    - generic resources load them from a ron file as an asset
    - Load resources from a RON file as an asset.
    - Resources should have some settings for generation (e.g. density, size, type).
    - Keep in mind that resources are just decoration for now
- [ ] better biomes:
    - I want to still have tile kinds like water and non-water (or maybe just land)
    - Then I want the biome to be a property of the tile, not a kind of tile.
    - For example we can color the tile based on the biome, e.g water that is in ocean biome is blue, water that is in river biome is light blue, water that is in lake biome is dark blue.
    - Then we can even look at resources based on biome: e.g tree's in grasland are simple trees, tree's in forest are big trees, tree's in desert are palm trees or cacti.
- [ ] water shader - Add a water shader to the hex map.
- [ ] vegetation shader - Add a vegetation shader to the hex map. (grass for grassland, dust for desert, leaves for forest: cute small details)
- [ ] hexmap_2d.rs - Render a 2D preview of the hex map.
    - Use a simple 2D rendering to display the hex map.
    - Display hex tiles with different colors based on their type (e.g. grass, water, mountain).
    - Add a simple camera movement to navigate the hex map.
    - Use `UP` and `DOWN` keys to zoom in and out of the hex map.
- [ ] hexmap_buildings.rs - Place buildings on hex tiles by clicking on them.
    - Use mouse input to place buildings on hex tiles.
    - Display buildings with different colors based on their type (e.g. red for houses, blue for factories).
    - Add a simple building placement algorithm that checks if the tile is valid for building (e.g. not occupied, not water).
    - Use `UP` and `DOWN` keys to switch between different building types.
    - Add the visibility shader for building placement, valid/invalid placement.
- [ ] hexmap_pathfinding.rs - Implement pathfinding on hex tiles.
    - Use A* or Dijkstra's algorithm to find paths between hex tiles.
    - Display the path on the hex map with a different color (e.g. green for path).
    - Use mouse input to select start and end tiles for pathfinding.
    - Add a simple pathfinding algorithm that checks if the tile is valid for pathfinding (e.g. not occupied, not water).
    - Add obstacles on the hex map that block pathfinding (e.g right click adds an obstacle).
- [ ] hexmap_save_load.rs - Save and load hex map data to/from a file.
    - Use a simple file format (e.g. JSON, YAML) to save hex map data.
    - Implement loading and saving of hex map data (e.g. height, temperature, resources, buildings).
    - Add a simple UI to save and load hex map data (e.g. buttons for save/load).
- [ ] hexmap_config.rs - Add a way to configure the settings for the hex map generation.
    - Figure out a few parameters that can be configured with nice names - the ones from Planet, but with better names and intuitive.
    - Add a simple UI to configure the settings (e.g. sliders, dropdowns).
    - Save and load the configuration to/from a file.
- [ ] factor out some library for re-use
