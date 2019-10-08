local GridProcessorSystem = class("GridProcessorSystem", System)

function GridProcessorSystem:update(dt)
  for i,v in pairs(self.targets) do
    local grid_inventory = v:get("GridInventory")
    local grid_processor = v:get("GridProcessor")

    grid_processor.timer = grid_processor.timer + dt
    if grid_processor.timer > grid_processor.process_delay then
      for output_name,output_data in pairs(grid_processor.resource_produced) do
        local inputs_available = true
        if datasets[output_name].requires ~= nil then
          for input_name,input_data in pairs(datasets[output_name].requires) do
            if grid_inventory.resources[input_name].count == 0 then
              inputs_available = false
            end
          end

          if inputs_available then
            if grid_inventory.resources[output_name].count < grid_inventory.resource_max_storage then
              grid_inventory.resources[output_name].count = grid_inventory.resources[output_name].count + 1
              for input_name,input_data in pairs(datasets[output_name].requires) do
                grid_inventory.resources[input_name].count = grid_inventory.resources[input_name].count - 1
              end
            end
          end
        end
      end
      grid_processor.timer = 0
    end
  end
end

function GridProcessorSystem:requires()
	return {"GridProcessor", "GridInventory"}
end

return GridProcessorSystem
