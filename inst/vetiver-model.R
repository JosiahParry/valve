library(vetiver)
cars_lm <- lm(mpg ~ ., data = mtcars)
v <- vetiver_model(cars_lm, "cars_linear")

library(pins)
# create folder
dir.create("inst/vetiver", recursive = TRUE)

# turn it into pin-board
b <- board_folder("inst/vetiver")

# write the model to the board
vetiver_pin_write(b, v)

# write the plumber api which references the board that has the model
vetiver_write_plumber(b, "cars_linear", file = "inst/vetiver/plumber.R")

vetiver_write_docker(v, path = "inst/vetiver")
