local TileSetGridAnimatorSystem = class("TileSetGridAnimatorSystem", System)

function TileSetGridAnimatorSystem:update(dt)
  for index, value in pairs(self.targets) do
    local tg = value:get("TileSetGrid")
    
    if tg.image_ref_final_frame ~= nil then
      if tg.animate_continuous then
        tg.animation_complete = false
      end 
      if tg.animation_complete == false then
        tg.elapsed_time = tg.elapsed_time + dt
        if tg.elapsed_time > tg.render_delay then
          if tg.image_ref < (tg.image_ref_initial_frame + tg.image_ref_final_frame - tg.image_ref_initial_frame) then
            tg.image_ref = tg.image_ref + 1          
          else
            tg.image_ref = tg.image_ref_initial_frame
            tg.animation_complete = true
            if tg.onelife then
              engine:removeEntity(value)
            end
          end
          tg.active_frame = tg.tileset.tiles[tg.image_ref]
          tg.elapsed_time = 0
        end
      else
        tg.active_frame = tg.tileset.tiles[tg.image_ref_initial_frame]
      end
    end
  end
end


function TileSetGridAnimatorSystem:requires()
	return {"TileSetGrid"}
end

return TileSetGridAnimatorSystem