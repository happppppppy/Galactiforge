local grid_functions = require("code/src/systems/grids/grid_functions")
local GridHeatSystem = class("GridHeatSystem", System)

function GridHeatSystem:update(dt)
  for index, value in pairs(self.targets) do
    local grid_item = value:get("GridItem")
    local grid_heat = value:get("GridHeat")
    local grid_inventory = value:get("GridInventory")
    if grid_heat.heat > 0 then

        if grid_inventory.resources["coolant"] ~= nil and grid_inventory.resources["coolant"].count > 0 then
            grid_heat.heat = grid_heat.heat - (1 * dt)--grid_inventory.resources["coolant"].efficiency) * dt --(grid_inventory.resources["coolant"].use_rate * grid_inventory.resources["coolant"].efficiency) * dt
            grid_inventory.resources["coolant"].count = grid_inventory.resources["coolant"].count - 1*dt -- grid_inventory.resources["coolant"].use_rate * dt
        end

      grid_heat.heat = grid_heat.heat - grid_heat.natural_cool_rate * dt 

    end
  end
end

function GridHeatSystem:requires()
	return {"GridItem", "GridHeat", "GridInventory"}
end

return GridHeatSystem
