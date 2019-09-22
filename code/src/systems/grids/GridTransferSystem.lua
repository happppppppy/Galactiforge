local GridTransferSystem = class("GridTransferSystem", System)

function GridTransferSystem:update(dt)
  for i,v in pairs(self.targets) do
    local parent = v:getParent()
    local grid_master = parent:get("GridMaster")

    local grid_item = v:get("GridItem")
    local grid_inventory = v:get("GridInventory")
    local grid_transfer = v:get("GridTransfer")
    
    local x_org = grid_master.grid_specs.allowed_grid.grid_origin.x
    local y_org = grid_master.grid_specs.allowed_grid.grid_origin.y


    --This could be optimised.
    grid_transfer.adjacent_grids = {}
    local adjacent_grid_right = grid_master.grid_items[y_org - grid_item.y][x_org - grid_item.x+1]
    local adjacent_grid_down = grid_master.grid_items[y_org - grid_item.y-1][x_org - grid_item.x]
    local adjacent_grid_left = grid_master.grid_items[y_org - grid_item.y][x_org - grid_item.x-1]
    local adjacent_grid_up = grid_master.grid_items[y_org - grid_item.y+1][x_org - grid_item.x]

    if adjacent_grid_right~= 0 and adjacent_grid_right:get("GridInventory") ~= nil then
      table.insert(grid_transfer.adjacent_grids, adjacent_grid_right:get("GridInventory"))
    end

    if adjacent_grid_down~= 0 and adjacent_grid_down:get("GridInventory") ~= nil then
      table.insert(grid_transfer.adjacent_grids, adjacent_grid_down:get("GridInventory"))
    end

    if adjacent_grid_left~= 0 and adjacent_grid_left:get("GridInventory") ~= nil then
      table.insert(grid_transfer.adjacent_grids, adjacent_grid_left:get("GridInventory"))
    end

    if adjacent_grid_up~= 0 and adjacent_grid_up:get("GridInventory") ~= nil then
      table.insert(grid_transfer.adjacent_grids, adjacent_grid_up:get("GridInventory"))
    end

    grid_transfer.timer = grid_transfer.timer + dt
    if grid_transfer.timer > grid_transfer.transfer_delay  and grid_transfer.adjacent_grids[grid_transfer.next_grid] ~= nil then
        for a,b in pairs(grid_transfer.adjacent_grids[grid_transfer.next_grid].resource_input) do
          if grid_inventory.resource_output[a] then
            if  b.count < b.max and grid_inventory.resource_output[a].count > 0 then
              b.count = b.count + 1
              grid_inventory.resource_output[a].count = grid_inventory.resource_output[a].count - 1
            end
          end
        end
      -- end
      if grid_transfer.next_grid < #grid_transfer.adjacent_grids then
        grid_transfer.next_grid = grid_transfer.next_grid + 1
      else
        grid_transfer.next_grid = 1
      end
      grid_transfer.timer = 0
    end
  end
end

function GridTransferSystem:requires()
	return {"GridTransfer", "GridInventory", "GridItem"}
end

return GridTransferSystem
