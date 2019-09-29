local FieryDeath = Component.create("FieryDeath")

function FieryDeath:initialize()
  self.animation_delay = 0.05
  self.elapsed_time = 0 
  self.count = 0
  self.max_count = 9
end