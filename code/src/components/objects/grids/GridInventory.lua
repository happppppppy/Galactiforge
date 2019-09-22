local GridInventory = Component.create("GridInventory")
function GridInventory:initialize(type)
  self.resource_input = {}
  for i,v in pairs(datasets[type].resource_input) do
    self.resource_input[i] = {}
    self.resource_input[i].count = v.count or 0
    self.resource_input[i].max = v.max or 0
  end

  self.resource_output = {}
  for i,v in pairs(datasets[type].resource_output) do
    self.resource_output[i] = {}
    self.resource_output[i].count = v.count or 0
    self.resource_output[i].max = v.max or 0
  end

  self.process_delay = datasets[type].process_delay or nil
end