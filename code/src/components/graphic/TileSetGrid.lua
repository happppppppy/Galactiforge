local TileSetGrid = Component.create("TileSetGrid")

function TileSetGrid:initialize(tileset, grid_type, grid_config, image_ref, image_ref_initial_frame, image_ref_final_frame, animated, animate_continuous, render_delay, onelife)
  if grid_config ~= nil then
    self.image_ref = grid_config[grid_type].image_ref or image_ref
    self.image_ref_initial_frame = grid_config[grid_type].image_ref_initial_frame or image_ref_initial_frame or nil
    self.image_ref_final_frame = grid_config[grid_type].image_ref_final_frame or image_ref_final_frame or nil
    self.animated = grid_config[grid_type].animated or animated or nil
    self.animate_continuous = grid_config[grid_type].animate_continuous or animate_continuous or false
    self.render_delay = grid_config[grid_type].render_delay or render_delay or nil  
    self.onelife = grid_config[grid_type].onelife or false
  else
    self.image_ref = image_ref
    self.image_ref_initial_frame = image_ref_initial_frame or nil
    self.image_ref_final_frame = image_ref_final_frame or nil
    self.animated = animated or false
    self.animate_continuous = animate_continuous or false
    self.render_delay = render_delay or 0.1
    self.onelife = onelife or false
  end

  self.tileset = tileset
  self.active_frame = self.tileset.tiles[self.image_ref]

  self.animation_complete = false
  self.elapsed_time = 0
  self.onelife = onelife or false
end