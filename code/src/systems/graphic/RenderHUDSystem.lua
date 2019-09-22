local RenderHUDSystem = class("RenderHUDSystem", System)

function RenderHUDSystem:draw()
  for index, value in pairs(self.targets) do
    local physics = value:get("PositionPhysics")
    local health = value:get("Health")
    local grid = value:get("Grid")

    local x_vel, y_vel = physics.body:getLinearVelocity( )
    local ship_velocity = math.sqrt(x_vel^2 + y_vel^2)
    love.graphics.setFont( font_hud )

    local x_loc = 20*(1/global_zoom_level)
    local y_loc = love.graphics.getHeight()*(1/global_zoom_level) - 20*(1/global_zoom_level)

    if lock_view_to_ship then
      x_loc = physics.body:getX()-love.graphics.getWidth()/2*(1/global_zoom_level) +20*(1/global_zoom_level)
      y_loc = physics.body:getY()+love.graphics.getHeight( )/2*(1/global_zoom_level) - 20*(1/global_zoom_level)
    end
    love.graphics.print("Selected: "..global_component_name_list[global_component_index], x_loc, y_loc-120*1/global_zoom_level, 0, 1/global_zoom_level, 1/global_zoom_level)
    love.graphics.print(string.format("Ship Velocity: %.2f ", ship_velocity), x_loc, y_loc-100*1/global_zoom_level, 0, 1/global_zoom_level, 1/global_zoom_level)
    love.graphics.print(string.format("X Coordinate: %.2f ", physics.body:getX()), x_loc, y_loc-80*1/global_zoom_level, 0, 1/global_zoom_level, 1/global_zoom_level)
    love.graphics.print(string.format("Y Coordinate: %.2f ", physics.body:getY()), x_loc, y_loc-60*1/global_zoom_level, 0, 1/global_zoom_level, 1/global_zoom_level)
    
    love.graphics.print(string.format("Ship Mass: %.2f ", physics.body:getMass()), x_loc, y_loc-20*1/global_zoom_level, 0, 1/global_zoom_level, 1/global_zoom_level)
    love.graphics.print(string.format("Ship Health: %d ", health.health), x_loc, y_loc, 0, 1/global_zoom_level, 1/global_zoom_level)

    love.graphics.print(string.format("FPS: %d ", love.timer.getFPS()), x_loc, y_loc-160*1/global_zoom_level, 0, 1/global_zoom_level, 1/global_zoom_level)
    love.graphics.print(string.format("DT: %d ", love.timer.getAverageDelta()), x_loc, y_loc-180*1/global_zoom_level, 0, 1/global_zoom_level, 1/global_zoom_level)
  end
end

function RenderHUDSystem:requires()
	return {"GridMaster", "Health", "PositionPhysics", "PlayerController"}
end

return RenderHUDSystem