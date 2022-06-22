# Webpack Stats

Webpack compilation statistics tooling. 

## CLI 

```
webpack-q stats.json list-entrypoints
entry-1:
  chunks:
  511
  603
  411
  
entry-2:
  chunks:
  553
  511
  603
```

```
# Output a graphviz dot file 
webpack-q stats.json traverse-entrypoint entry-1 -f dot

#output json
webpack-q stats.json traverse-entrypoint entry-1 -f json

#output html viz with d3
webpack-q stats.json traverse-entrypoint entry-1 -f html
```




## Reference JSON

The `test_projects` file contains relevant reference for each webpack stats file.
