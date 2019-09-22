local GridProcessor = Component.create("GridProcessor")
function GridProcessor:initialize(type)
  self.process_delay = datasets[type].process_delay or nil
  self.timer = 0
end