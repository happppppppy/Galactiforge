local grid_functions = require("code/src/systems/grids/grid_functions")
local GridHeatSystem = class("GridHeatSystem", System)

function GridHeatSystem:update(dt)
  for index, value in pairs(self.targets) do
    local grid_item = value:get("GridItem")
    local grid_heat = value:get("GridHeat")
    local grid_inventory = value:get("GridInventory")

    if grid_heat.heat > 0 then
      local resources_found, resources_available = grid_functions:getResourceAvailable(grid_inventory)

      if resources_available then
        for _,v in pairs(resources_found) do
          -- if grid_inventory.resource_input[v].count > 0 then
          --   grid_inventory.resource_input[v].consumed = grid_inventory.resource_input[v].consumed + grid_inventory.resource_input[v].use_rate * dt
          --   grid_heat.heat = grid_heat.heat - grid_heat.natural_cool_rate * dt - grid_inventory.resource_input[v].efficiency * dt
          -- end

        end
      else
        grid_heat.heat = grid_heat.heat - grid_heat.natural_cool_rate * dt 
      end
    end
  end
end

function GridHeatSystem:requires()
	return {"GridItem", "GridHeat", "GridInventory"}
end

return GridHeatSystem
