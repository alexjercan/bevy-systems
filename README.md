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
- [ ] biome_classification.rs - Classify hex tiles into biomes based on noise values.
    - Use a simple classification algorithm to assign biomes (e.g. grassland, desert, forest) based on height and temperature.
    - Display hex tiles with different colors based on their biome (e.g yellow for desert, green for grassland, dark green for forest).
- [ ] hexmap_resources.rs - Generate resources on hex tiles based on noise values.
    - Compute density of resources based on the noise values (temperature, height, moisture, etc.)
    - Use hexmap information to generate resource density maps (e.g. tree, ore, food).
    - Display resources on hex tiles with colors representing resource density (e.g. green for trees, gray for ore, yellow for food).
    - Switch between different resource types using `UP` and `DOWN` keys.
    - Add a threshold metter to filter out low-density resources and display only high-density resources.
    - Maybe add a noisemap for resources, to add more randomness to the resource distribution.
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
