local TileSetGrid = Component.create("TileSetGrid")

function TileSetGrid:initialize(component_values, tileset)

  self.image_ref = component_values.image_ref or nil
  self.image_ref_initial_frame = component_values.image_ref_initial_frame or nil
  self.image_ref_final_frame = component_values.image_ref_final_frame or nil
  self.animated = component_values.animated or false
  self.animate_continuous = component_values.animate_continuous or false
  self.render_delay = component_values.render_delay or nil  
  self.onelife = component_values.onelife or false

  self.tileset = tileset
  self.active_frame = self.tileset.tiles[self.image_ref]

  self.animation_complete = false
  self.elapsed_time = 0
end