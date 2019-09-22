local GridConsumer = Component.create("GridConsumer")
function GridConsumer:initialize(in_or_out)
  self.count_used = 0
  self.in_or_out = in_or_out
end