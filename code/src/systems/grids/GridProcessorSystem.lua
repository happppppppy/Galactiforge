local GridProcessorSystem = class("GridProcessorSystem", System)

function GridProcessorSystem:update(dt)
  for i,v in pairs(self.targets) do
    local grid_inventory = v:get("GridInventory")
    local grid_processor = v:get("GridProcessor")

    grid_processor.timer = grid_processor.timer + dt
    if grid_processor.timer > grid_processor.process_delay then
      for _,output in pairs(grid_processor.resource_produced) do
        local inputs_available = true
        for _,input in pairs(datasets[output].requires) do
          if grid_inventory.resources[input].count == 0 then
            inputs_available = false
          end
        end

        if inputs_available then
          grid_inventory.resources[output].count = grid_inventory.resources[output].count + 1
          for _,input in pairs(datasets[output].requires) do
            grid_inventory.resources[input].count = grid_inventory.resources[input].count - 1
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
