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
    
    grid_transfer.adjacent_grids = {}
    for row=1,#grid_transfer.transfer_grid,1 do
      for col=1,#grid_transfer.transfer_grid[row],1 do
        if grid_transfer.transfer_grid[row][col] ~= 0 then
          if grid_master.grid_items[y_org - grid_item.y + row-2] ~= nil and grid_master.grid_items[y_org - grid_item.y + row-2][x_org - grid_item.x + col-2] ~= nil and grid_master.grid_items[y_org - grid_item.y + row-2][x_org - grid_item.x + col-2] ~= 0 then
            local entity = grid_master.grid_items[y_org - grid_item.y + row-2][x_org - grid_item.x + col-2]
            if grid_master.grid_items[y_org - grid_item.y + row-2][x_org - grid_item.x + col-2]:get("GridInventory") ~= nil then
              table.insert(grid_transfer.adjacent_grids, grid_master.grid_items[y_org - grid_item.y + row-2][x_org - grid_item.x + col-2]:get("GridInventory"))
            end
          end
        end
      end
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
