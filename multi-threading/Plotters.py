import os
import matplotlib.pyplot as plt
import numpy as np

f = open("Vector.txt", "r")

numbers = f.read().split(",")[:-1]
numbers = [int(x) for x in numbers]
print(len(numbers))
print(numbers)
plt.hist(numbers, bins=30, density=True)
plt.ylabel('Probability')
plt.xlabel('Data');

plt.show()


