# bevy-single-variable-function-mesh

This crate creates procedurally generated 2D or 3D bevy meshes by using
single-variable functions.

Warning: This project is in an very early stage and a lot of code is missing.

## Build

```
cargo r
```

## Examples

<table>

<tr><th>Source code</th><th>Preview</th></td>

<tr>
<td>
<pre>
fn squircle(x: f32) -> f32 {
    (1.0 - (x).abs().powf(4.0)).powf(0.25)
}
</pre>
</td>
<td><img src="images/squircle.png"></td>
</tr>

</table>
