local PlayerControllerSystem = class("PlayerControllerSystem", System)

function PlayerControllerSystem:update(dt)
	for index, value in pairs(self.targets.pool1) do
		local weapon = value:get("Weapon")
		local grid_item = value:get("GridItem")

		local parent = value:getParent()
		local physics = parent:get("PositionPhysics")

		if weapon.aimed then
			mouse_x, mouse_y = love.graphics.inverseTransformPoint( love.mouse.getPosition() )
			grid_item.t_pos_grid_physics = math.atan2(mouse_y - grid_item.y_pos_grid_physics, mouse_x - grid_item.x_pos_grid_physics) + math.rad(90)
			grid_item.t_render = grid_item.t_pos_grid_physics - physics.body:getAngle()
		end

		--Fire control updates      
		if (weapon.activation == "mouse1" and love.mouse.isDown(1)) or love.keyboard.isDown(weapon.activation)  then
			eventmanager:fireEvent(FireEvent(value, dt))
		end
	end
end

function PlayerControllerSystem:requires()
	return {pool1 = {"PlayerController", "Weapon"}}
end

return PlayerControllerSystem