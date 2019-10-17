local GridTransfer = Component.create("GridTransfer")
function GridTransfer:initialize(component_values)
  self.transfer_delay = component_values.transfer_rate
  self.transfer_grid = component_values.transfer_grid

  self.timer = 0
  self.adjacent_grids = {}
  self.next_grid = 1
end