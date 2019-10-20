local GridProcessorSystem = class("GridProcessorSystem", System)

function GridProcessorSystem:update(dt)
  for i,v in pairs(self.targets) do
    local grid_inventory = v:get("GridInventory")
    local grid_processor = v:get("GridProcessor")

    grid_processor.timer = grid_processor.timer + dt
    if grid_processor.timer > grid_processor.process_delay then
      for output_name,output_data in pairs(grid_processor.resource_produced) do
        if output_data.active then
          if datasets[output_name].requires ~= nil then
            for input_name,input_data in pairs(datasets[output_name].requires) do
              if grid_inventory.resources[input_name].count == 0 then
                goto NEXTRESOURCE
              end
            end
            if grid_inventory.resources[output_name].count < grid_inventory.max_storage then
              grid_inventory.resources[output_name].count = grid_inventory.resources[output_name].count + 1
              for input_name,input_data in pairs(datasets[output_name].requires) do
                grid_inventory.resources[input_name].count = grid_inventory.resources[input_name].count - 1
              end
            end
          end
          ::NEXTRESOURCE::
        end
      end
      grid_processor.timer = 0
    end
  end
end

function GridProcessorSystem:fireEvent(event)
  local grid_master = event.parent:get("GridMaster")
  local grid = grid_master.grid_items[grid_master.grid_specs.allowed_grid.grid_origin.y - event.y_loc][grid_master.grid_specs.allowed_grid.grid_origin.x - event.x_loc]
  if grid ~= 0 then
    local grid_processor = grid:get("GridProcessor")
    if grid_processor ~= nil then
        grid_processor.resource_produced[grid_processor.resource_names[grid_processor.next_resource]].active = false
        if grid_processor.next_resource < #grid_processor.resource_names then
          grid_processor.next_resource = grid_processor.next_resource + 1
        else
          grid_processor.next_resource = 1
        end
        grid_processor.resource_produced[grid_processor.resource_names[grid_processor.next_resource]].active = true
      -- end
    end
  end
end

function GridProcessorSystem:requires()
	return {"GridProcessor", "GridInventory"}
end

return GridProcessorSystem
