local GameObjectiveSystem = class("GameObjectiveSystem", System)
local timer = 0
local end_time = 5
function GameObjectiveSystem:update(dt)
	for index, value in pairs(self.targets.pool1) do
		objective = value:get("DestroyOpponentsObjective")
		timer = timer + dt
		for i,v in pairs(objective.ships) do
			if timer > end_time then
				engine:removeEntity(v, true)
				table.remove(objective.ships, i)
				timer = 0
			end
			if #objective.ships == 0 then
				print("Winner winner chicken dinner")
				engine:removeEntity(value, true)
			end
		end
	end

end

function GameObjectiveSystem:requires()
	return {pool1 = {"DestroyOpponentsObjective"}}
end

return GameObjectiveSystem