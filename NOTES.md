# LinuxCNC tolerance modes

## `G64`

Default is said to be `G64` with no tolerance (as per
[here](https://www.forum.linuxcnc.org/20-g-code/44022-understanding-g64-behavior)) which means

> keep the best speed possible, no matter how far away from the programmed point you end up.
>
> <http://linuxcnc.org/docs/html/gcode/g-code.html#gcode:g64>

Although the LinuxCNC source dode _might_ set it to some value?

`G64 P0.01` (0.01 machine unit max deviation) will slow down before reaching the end point so the
acceleration can keep up to not allow deviation beyond the set threshold. This results in a rounded
corner.

## `G61` Exact path mode

> Moves will slow or stop as needed to reach every programmed point. If two sequential moves are
> exactly co-linear movement will not stop.
>
> <http://linuxcnc.org/docs/html/gcode/g-code.html#gcode:g61>

## `G61.1` Exact stop mode

> movement will stop at the end of each programmed segment.
>
> <http://linuxcnc.org/docs/html/gcode/g-code.html#gcode:g61.1>

Not sure why this is necessary but it's easy enough to add I think.
