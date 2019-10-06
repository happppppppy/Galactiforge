local GridConsumerSystem = class("GridConsumerSystem", System)

function GridConsumerSystem:update(dt)
  for i,v in pairs(self.targets) do
    local grid_inventory = v:get("GridInventory") 
    local grid_consumer = v:get("GridConsumer")

    -- -- if grid_consumer.count_used > 0 then
    --   -- if grid_consumer.in_or_out == "input" then
    --     for a,b in pairs(grid_inventory.resource_input) do
    --       b.count = b.count - b.consumed
    --       b.consumed = 0
    --     end
    --   -- elseif grid_consumer.in_or_out == "output" then
    --     for a,b in pairs(grid_inventory.resource_output) do
    --       b.count = b.count - b.consumed
    --       b.consumed = 0
    --     end
    --   -- end
    --   -- grid_consumer.count_used = 0
    -- -- end

  end
end

function GridConsumerSystem:requires()
	return {"GridConsumer", "GridInventory", "GridItem"}
end

return GridConsumerSystem
