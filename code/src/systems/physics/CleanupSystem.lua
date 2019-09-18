local CleanupSystem = class("CleanupSystem", System)

function CleanupSystem:onRemoveEntity(entity, group)
  local physics = entity:get("PositionPhysics") or entity:get("GridItem")
  if physics.fixture ~= nil then
    physics.fixture:destroy()
  end
  if physics.shape ~= nil then
    physics.shape:release()
  end
  if physics.body ~= nil then
    physics.body:release()
  end
end

function CleanupSystem:requires()
	return {pool1={"PositionPhysics"}, pool2={"GridItem"}}
end

return CleanupSystem