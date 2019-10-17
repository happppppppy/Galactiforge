local GridTransferSystem = class("GridTransferSystem", System)

function GridTransferSystem:update(dt)
  for i,v in pairs(self.targets) do
    local grid_master = v.parent:get("GridMaster")
    local grid_item = v:get("GridItem")
    local grid_inventory = v:get("GridInventory")
    local grid_transfer = v:get("GridTransfer")

    local x_offset = grid_transfer.transfer_grid[grid_transfer.next_grid].o_x
    local y_offset = grid_transfer.transfer_grid[grid_transfer.next_grid].o_y

    local x_org = grid_master.grid_specs.allowed_grid.grid_origin.x
    local y_org = grid_master.grid_specs.allowed_grid.grid_origin.y

    if grid_master.grid_items[y_org - grid_item.y + y_offset] ~= nil and grid_master.grid_items[y_org - grid_item.y + y_offset][x_org - grid_item.x + x_offset] ~= nil then
      local target_grid = grid_master.grid_items[y_org - grid_item.y + y_offset][x_org - grid_item.x + x_offset]
      if target_grid ~= 0 then
        local target_inventory = target_grid:get("GridInventory")
        grid_transfer.timer = grid_transfer.timer + dt
        if target_inventory ~= nil and grid_transfer.timer > grid_transfer.transfer_delay then
          for resource_name, resource_details in pairs(target_inventory.resources) do
            if grid_inventory.resources[resource_name] ~= nil and grid_inventory.resources[resource_name].count > 0 and resource_details.count < target_inventory.max_storage then
              grid_inventory.resources[resource_name].count = grid_inventory.resources[resource_name].count -1
              resource_details.count = resource_details.count + 1
            end
          end
        end    
        if grid_transfer.next_grid < #grid_transfer.transfer_grid then
          grid_transfer.next_grid = grid_transfer.next_grid + 1
        else
          grid_transfer.next_grid = 1
        end
        grid_transfer.timer = 0
      end

    else
      if grid_transfer.next_grid < #grid_transfer.transfer_grid then
        grid_transfer.next_grid = grid_transfer.next_grid + 1
      else
        grid_transfer.next_grid = 1
      end
    end
  end
end

function GridTransferSystem:requires()
	return {"GridTransfer", "GridInventory", "GridItem"}
end

return GridTransferSystem
