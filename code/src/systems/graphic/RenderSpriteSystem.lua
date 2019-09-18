local RenderSpriteSystem = class("RenderSpriteSystem", System)

function RenderSpriteSystem:draw()
    for index, value in pairs(self.targets) do
      local sprite = value:get("Sprite")
      local physics = value:get("PositionPhysics")

      love.graphics.draw(sprite.image, physics.body:getX(), physics.body:getY(), physics.body:getAngle(), sprite.sx, sprite.sy, sprite.ox, sprite.oy)
    end
end

function RenderSpriteSystem:requires()
	return {"Sprite", "PositionPhysics"}
end

return RenderSpriteSystem