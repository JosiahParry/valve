# valve 0.1.1.9000


* Adds `n_min` argument (default 1). Specifies the minimum number of plumber APIs always running. Previously, some requests might fail if all connections had gone stale and been pruned. 
  - Valve will now always spawn the first connection automatically. Additional connections will be spawned on-demand. Once `n_min` has been reached, the number of connections will never be lower. 
