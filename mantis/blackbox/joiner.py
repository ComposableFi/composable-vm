# this is up to take production data in heterogenous shape, and force it into pandas dataframe(s)
import numpy as np

a = np.array([1, 2, 3, 3])
b = np.array([4, 5, 6, 42])

print(b[a > 2])
print(np.cos(b))