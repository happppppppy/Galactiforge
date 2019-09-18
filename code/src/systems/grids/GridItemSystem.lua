local grid_functions = require("code/src/systems/grids/grid_functions")

local GridItemSystem = class("GridItemSystem", System)

local transfer_time = 0
local transfer_rate = 0.05

function updateNeighbours(entity)
  local all_entities = engine:getEntitiesWithComponent("GridItem")
  for _, primary_entity in pairs(all_entities) do
    primary_grid_value = primary_entity:get("GridItem")
    primary_grid_value.grids = {}
    primary_grid_value.next_neighbour = 1
    for _, secondary_entity in pairs(all_entities) do
      if primary_entity:getParent() == secondary_entity:getParent() then
        secondary_grid_value = secondary_entity:get("GridItem")
        for row = 1, #primary_grid_value.transfer_grid do
          for col = 1, #primary_grid_value.transfer_grid[row] do    
            if primary_grid_value.transfer_grid[row][col] == 1 then
              if primary_grid_value.x - secondary_grid_value.x - (row-2) == 0 and primary_grid_value.y - secondary_grid_value.y - (col-2) == 0 then
                primary_grid_value.grids[#primary_grid_value.grids+1] = secondary_grid_value
              end
            end
          end
        end
      end
    end
  end
end

function GridItemSystem:onRemoveEntity(entity)
  local grid_item = entity:get("GridItem")
  local parent = entity:getParent()
  local ship = parent:get("Ship")

  ship.grid_status[ship.ship_specs.allowed_grid.grid_origin.y - grid_item.y][ship.ship_specs.allowed_grid.grid_origin.x - grid_item.x] = 0

  updateNeighbours(entity)
end

function GridItemSystem:onAddEntity(entity)
  updateNeighbours(entity)
end

function GridItemSystem:update(dt)
  for index, value in pairs(self.targets) do
    --Resource updates

    local grid_item = value:get("GridItem")
    local tileset = value:get("TileSetGrid")

    
    -- -- Process or use resources
    grid_item.resources_available = true

    for res_in_index,res_in_value in pairs(grid_item.resource_input) do
      if grid_item.resource_input[res_in_index].count <= 0 then
        grid_item.resources_available = false
      end
    end

    if grid_item.resources_used_count > 0 or grid_item.auto_increment then
      grid_item.process_time = grid_item.process_time + dt 
      if grid_item.use_instantaneously then
        grid_item.process_time = grid_item.process_delay + 1
      end

      local process_ok = false
      if grid_item.process_time > grid_item.process_delay and grid_item.resources_available then
        for res_out_index, res_out_value in pairs(grid_item.resource_output) do
          if grid_item.resource_output[res_out_index].count < grid_item.resource_output[res_out_index].max then 
            process_ok = true
          end
        end
        if #grid_item.resource_output == 0 then process_ok = true end
        if grid_item.type == "large_thruster" then print("Hello", #grid_item.resource_output, process_ok) end

        if process_ok then
          for res_in_index,res_in_value in pairs(grid_item.resource_input) do
            grid_item.resource_input[res_in_index].count = grid_item.resource_input[res_in_index].count -  grid_item.resource_input[res_in_index].use_rate
            if not grid_item.auto_increment then
              grid_item.resources_used_count =  grid_item.resources_used_count - 1
            end
          end
          for res_out_index, res_out_value in pairs(grid_item.resource_output) do
              grid_item.resource_output[res_out_index].count = grid_item.resource_output[res_out_index].count + 1
          end
        end
        grid_item.process_time = 0
      end
    end

    --Transfer resources
    transfer_time = transfer_time + dt
    if transfer_time > transfer_rate and #grid_item.grids > 0 then
     
      local all_inputs_available = true
      for resource_name,resource_item in pairs(grid_item.grids[grid_item.next_neighbour].resource_input) do
        if grid_item.resource_output[resource_name] == nil or grid_item.resource_output[resource_name].count == 0 or resource_item.count >= resource_item.max then
          all_inputs_available = false
        end
      end

      if all_inputs_available then
        for _, v in pairs(grid_item.grids[grid_item.next_neighbour].resource_input) do
          v.count = v.count + 1
        end

        for _, v in pairs(grid_item.resource_output) do
          v.count = v.count - 1
        end
      end
      
      if grid_item.next_neighbour < #grid_item.grids then
        grid_item.next_neighbour = grid_item.next_neighbour + 1
      else
        grid_item.next_neighbour = 1
      end

      transfer_time = 0
    end

    --Render updates
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    local tileset = value:get("TileSetGrid")

    grid_item.t_render, grid_item.x_render, grid_item.y_render = grid_functions:getRenderPositions(grid_item.x, grid_item.y, grid_item.grid_scale, tileset, physics, 0, 0)

    if grid_item.flag_for_removal then
      engine:removeEntity(value)
    end

  end
end

function GridItemSystem:requires()
	return {"GridItem", "TileSetGrid"}
end

return GridItemSystem
