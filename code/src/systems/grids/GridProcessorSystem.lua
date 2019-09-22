local GridProcessorSystem = class("GridProcessorSystem", System)

function GridProcessorSystem:update(dt)
  for i,v in pairs(self.targets) do
    local grid_inventory = v:get("GridInventory")
    local grid_processor = v:get("GridProcessor")

    grid_processor.timer = grid_processor.timer + dt
    if grid_processor.timer > grid_processor.process_delay then
      local resources_available = true
      for a,b in pairs(grid_inventory.resource_input) do
        if b.count <= 0 then
          resources_available = false
        end
      end
      if resources_available then
        for a,b in pairs(grid_inventory.resource_input) do
          b.count = b.count -1
        end
        for a,b in pairs(grid_inventory.resource_output) do
          b.count = b.count + 1
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
