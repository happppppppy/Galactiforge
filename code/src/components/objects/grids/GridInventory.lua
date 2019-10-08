local GridInventory = Component.create("GridInventory")
function GridInventory:initialize(type)
  self.resources = {}

  for output_name,output_data in pairs(datasets[type].resource_produced) do
    self.resources[output_name] = {}
    self.resources[output_name].use_rate = output_data.use_rate or 1
    self.resources[output_name].count = output_data.initial_count or 0
    self.resources[output_name].type = "output"

    if datasets[output_name].requires ~= nil then
      for input_name,input_data in pairs(datasets[output_name].requires) do
        self.resources[input_name] = {}
        self.resources[input_name].use_rate = input_data.use_rate or 1
        self.resources[input_name].count = input_data.initial_count or 0
        self.resources[input_name].type = "input"
      end
    end
  end

  for consumed_name,consumed_data in pairs(datasets[type].resource_consumed) do
    self.resources[consumed_name] = {}
    self.resources[consumed_name].use_rate = consumed_data.use_rate or 1
    self.resources[consumed_name].count = consumed_data.initial_count or 0
    self.resources[consumed_name].type = "stored"
    self.resources[consumed_name].efficiency = consumed_data.efficiency or 1
  end
  
  self.process_delay = datasets[type].process_delay or nil
  self.resource_max_storage = datasets[type].resource_max_storage or 0
end