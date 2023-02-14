## Why Autometrics?

### Metrics today are hard to use

Metrics are a powerful and relatively inexpensive tool for understanding your system in production.

However, they are still hard to use. Developers need to:
- Think about what metrics to track and which metric type to use (counter, histogram... 😕)
- Figure out how to write PromQL or another query language to get some data 😖
- Verify that the data returned actually answers the right question 😫

### Simplifying code-level observability

Many modern observability tools promise to make life "easy for developers" by automatically instrumenting your code with an agent or eBPF. Others ingest tons of logs or traces -- and charge high fees for the processing and storage.

Most of these tools treat your system as a black box and use complex and pricey processing to build up a model of your system. This, however, means that you need to map their model onto your mental model of the system in order to navigate the mountains of data.

Autometrics takes the opposite approach. Instead of throwing away valuable context and then using compute power to recreate it, it starts inside your code. It enables you to understand your production system at one of the most fundamental levels: from the function.

### Standardizing function-level metrics

Functions are one of the most fundamental building blocks of code. Why not use them as the building block for observability?

A core part of Autometrics is the simple idea of using standard metric names and a consistent scheme for tagging/labeling metrics. The three metrics currently used are: `function.calls.count`, `function.calls.duration`, and `function.calls.concurrent`.

### Labeling metrics with useful, low-cardinality function details

The following labels are added automatically to all three of the metrics: `function` and `module`.

For the function call counter, the following labels are also added:

- `caller` - (see ["Tracing Lite"](#tracing-lite) below)
- `result` - either `ok` or `error` if the function returns a `Result`
- `ok` / `error` - see the next section

#### Static return type labels

If the concrete `Result` types implement `Into<&'static str>`, the that string will also be added as a label value under the key `ok` or `error`.

For example, you can have the variant names of your error enum included as labels:
```rust
use strum::IntoStaticStr;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum MyError {
  SomethingBad(String),
  Unknown,
  ComplexType { message: String },
}
```
In the above example, functions that return `Result<_, MyError>` would have an additional label `error` added with the values `something_bad`, `unknown`, or `complex_type`.

This is more useful than tracking external errors like HTTP status codes because multiple logical errors might map to the same status code.

Autometrics only supports `&'static str`s as labels to avoid the footgun of attaching labels with too many possible values. The [Prometheus docs](https://prometheus.io/docs/practices/naming/#labels) explain why this is important in the following warning:

> CAUTION: Remember that every unique combination of key-value label pairs represents a new time series, which can dramatically increase the amount of data stored. Do not use labels to store dimensions with high cardinality (many different label values), such as user IDs, email addresses, or other unbounded sets of values.

### "Tracing Lite"

A slightly unusual idea baked into autometrics is that by tracking more granular metrics, you can debug some issues that we would traditionally need to turn to tracing for.

Autometrics can be added to any function in your codebase, from HTTP handlers down to database methods.

This means that if you are looking into a problem with a specific HTTP handler, you can browse through the metrics of the functions _called by the misbehaving function_.

Simply hover over the function names of the nested function calls in your IDE to look at their metrics. Or, you can directly open the chart of the request or error rate of all functions called by a specific function.
