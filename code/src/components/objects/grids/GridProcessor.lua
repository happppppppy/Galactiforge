local GridProcessor = Component.create("GridProcessor")
function GridProcessor:initialize(component_values)
  self.resource_produced = {}
  for resource_name, resource_values in pairs(component_values.resource_produced) do
    self.resource_produced[resource_name] = {}
    self.resource_produced[resource_name].use_rate = resource_values.use_rate
  end

  self.process_delay = component_values.process_rate
  self.timer = 0

end