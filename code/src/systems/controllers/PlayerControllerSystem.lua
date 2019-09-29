local PlayerControllerSystem = class("PlayerControllerSystem", System)

function PlayerControllerSystem:requires()
	return {"PlayerController"}
end

return PlayerControllerSystem