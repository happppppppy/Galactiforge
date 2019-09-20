local grid_functions = require("code/src/systems/grids/grid_functions")

local GridItemSystem = class("GridItemSystem", System)

local transfer_time = 0
local transfer_rate = 0.05

function updateNeighbours(entity)
  parent = entity:getParent()
  grid_master = parent:get("GridMaster")
  all_grids = grid_master.grids
  for _,k in pairs(all_grids) do
    if k.grid_inventory ~= nil then
      for _,v in pairs(k.grid_inventory.transfer_grid) do
        if v.usable == 1 then
          for _, secondary_grid in pairs(grid_master.grids) do
            if k.grid_item.x + v.x == secondary_grid.grid_item.x and k.grid_item.y + v.y == secondary_grid.grid_item.y then
              v.grid_reference = secondary_grid.grid_inventory
              break
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
  local grid_master = parent:get("GridMaster")

  grid_master.grid_status[grid_master.grid_specs.allowed_grid.grid_origin.y - grid_item.y][grid_master.grid_specs.allowed_grid.grid_origin.x - grid_item.x] = 0
  if entity:get("GridInventory") then
    updateNeighbours(entity)
  end
end

function GridItemSystem:onAddEntity(entity)
  if entity:get("GridInventory") then
    updateNeighbours(entity)
  end
end

function GridItemSystem:update(dt)
  for index, value in pairs(self.targets) do
    --Resource updates

    local grid_item_inventory = value:get("GridInventory")
    local tileset = value:get("TileSetGrid")

    if grid_item_inventory ~= nil then
      -- -- Process or use resources
      grid_item_inventory.resources_available = true

      for res_in_index,res_in_value in pairs(grid_item_inventory.resource_input) do
        if grid_item_inventory.resource_input[res_in_index].count <= 0 then
          grid_item_inventory.resources_available = false
        end
      end

      if grid_item_inventory.resources_used_count > 0 or grid_item_inventory.auto_increment then
        grid_item_inventory.process_time = grid_item_inventory.process_time + dt 
        if grid_item_inventory.use_instantaneously then
          grid_item_inventory.process_time = grid_item_inventory.process_delay + 1
        end

        local process_ok = false
        if grid_item_inventory.process_time > grid_item_inventory.process_delay and grid_item_inventory.resources_available then
          for res_out_index, res_out_value in pairs(grid_item_inventory.resource_output) do
            if grid_item_inventory.resource_output[res_out_index].count < grid_item_inventory.resource_output[res_out_index].max then 
              process_ok = true
            end
          end
          if #grid_item_inventory.resource_output == 0 then process_ok = true end
          if process_ok then
            for res_in_index,res_in_value in pairs(grid_item_inventory.resource_input) do
              grid_item_inventory.resource_input[res_in_index].count = grid_item_inventory.resource_input[res_in_index].count -  grid_item_inventory.resource_input[res_in_index].use_rate
              if not grid_item_inventory.auto_increment then
                grid_item_inventory.resources_used_count =  grid_item_inventory.resources_used_count - 1
              end
            end
            for res_out_index, res_out_value in pairs(grid_item_inventory.resource_output) do
                grid_item_inventory.resource_output[res_out_index].count = grid_item_inventory.resource_output[res_out_index].count + 1
            end
          end
          grid_item_inventory.process_time = 0
        end
      end

      local resources_available = true
      for i,v in pairs(grid_item_inventory.resource_output) do
        if v.count <= 0 then
          v.available = false
        else
          v.available = true
        end
      end


      --Transfer resources
      for i = 1, #grid_item_inventory.transfer_grid,1 do
        if grid_item_inventory.transfer_grid[i].grid_reference ~= nil then
          for i,v in pairs(grid_item_inventory.transfer_grid[i].grid_reference.resource_input) do 
            if v.count < v.max then v.count = v.count + 1 end
          end
        end
      end

      
    end

    --Render updates
    local parent = value:getParent()
    local physics = parent:get("PositionPhysics")
    local tileset = value:get("TileSetGrid")
    local grid_item = value:get("GridItem")

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
