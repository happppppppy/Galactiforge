local GridTransfer = Component.create("GridTransfer")
function GridTransfer:initialize(type)
  self.transfer_delay = datasets[type].process_delay or nil
  self.timer = 0
  self.adjacent_grids = {}
  self.next_grid = 1
end