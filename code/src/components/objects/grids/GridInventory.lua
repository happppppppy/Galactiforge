local GridInventory = Component.create("GridInventory")
function GridInventory:initialize(type)

  self.resources = {}
  for _,output in pairs(datasets[type].resource_produced) do
      self.resources[output] = {}
      self.resources[output].count = 0
      self.resources[output].type = "output"

    for _,input in pairs(datasets[output].requires) do
      self.resources[input] = {}
      self.resources[input].count = 10
      self.resources[input].type = "input"
    end
  end

  for _,consumed in pairs(datasets[type].resource_consumed) do
    self.resources[consumed] = {}
    self.resources[consumed].count = 10
    self.resources[consumed].type = "stored"
  end
  
  self.process_delay = datasets[type].process_delay or nil
end