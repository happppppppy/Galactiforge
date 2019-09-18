local TileSet = Component.create("TileSet")

function TileSet:initialize(imagePath, num_tiles, tile_width, tile_height, animation_delay, onelife)
  self.image = love.graphics.newImage(imagePath)
  self.tiles = {}
  for i=num_tiles,1,-1 do
    self.tiles[i] = love.graphics.newQuad(tile_width * (i-1), 0, tile_width, tile_height, self.image:getDimensions())
  end
  self.current_frame = 1
  self.active_frame = self.tiles[self.current_frame]
  self.animation_delay = animation_delay
  self.num_tiles = num_tiles
  self.fire = false
  self.elapsed_time = 0
  self.tile_width  = tile_width
  self.tile_height = tile_height
  self.draw = true
  self.onelife = onelife
  self.drawn = false
end