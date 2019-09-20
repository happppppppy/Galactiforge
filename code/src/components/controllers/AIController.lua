local AIController = Component.create("AIController")

function AIController:initialize()
  self.target_direction = 0
  self.target_distance = 0

  self.range = 300
end