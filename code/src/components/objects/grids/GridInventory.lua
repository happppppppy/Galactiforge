local GridInventory = Component.create("GridInventory")
function GridInventory:initialize(component_values)
  self.resources = {}
  for resource_name, resource_value in pairs(component_values.whitelist) do
    self.resources[resource_name] = {}
    self.resources[resource_name].count = resource_value.count or 1
    self.resources[resource_name].type = resource_value.type or nil
  end

  self.process_delay = component_values.process_rate
  self.max_storage = component_values.max_storage
end