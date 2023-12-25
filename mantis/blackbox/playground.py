import numpy as np
a = np.array([1, 2, 3, 3])
b = np.array([4, 5, 6, 42])

print(b[a > 2])
print(np.cos(b))


import cvxpy as cp

x  = cp.Variable(3)
y = cp.Variable(2)

t = np.array([8, 12, 16, 9, 32])
c = np.array([2000, 1000, 900, 1500,500])

constraints = [x.sum() >= 0.9999, y.sum() >= 0.9999, x.sum() <=1,  y.sum() <= 1, x >= 0, x <=1, y >=0 , y <=1 ]


obj = cp.Minimize(c[0:3] @ x + c[3:5] @ y)
                  
problem = cp.Problem(obj, constraints)
result = problem.solve()
print(result)
print(problem.status)
print("value : ", problem.value)
print(problem.solution)