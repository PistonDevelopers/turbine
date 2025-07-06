# Turbine-Process3D

Processing 3D often does not require soft realtime constraints,
but used as part of content production pipelines.

For example, in 3D animation, high quality of final output,
instant feedback during design and animation,
plus determinism for distributed computing,
is more important than performance.

When people design a game engine, they often focus on the science of the core engine itself
and forget that most of the work that goes into most games is in the content production, not coding.
Processing can easily become a bottleneck, not due to raw performance,
but because of inflexible architecture.

For example, by using Bounded Volume Hierarchy (BVH) for ray tracing,
one is immediately locked down to a particular architecture that can be limiting later on.

Processing often requires analyzing the data and exploiting the knowledge about it.
A choice of architecture can make it hard to reason about the data.
If you can not reason about the data, then you can not reason about the problem.

Customization of 3D processing is often desirable because one does not need a complete engine or framework,
but only enough code to solve the problem and integrate it in a larger content production pipeline.

So, how do you solve customization of 3D processing by design?

By using run-length compression on masks per tile over a triangle list,
one can solve 90% of performance issues for customized rendering,
without sacrificing flexibility or constraining oneself to a choice of architecture.
Instead of solving rendering as an end-to-end problem, one can focus on solving the preparation stage only.
Let people design their own algorithms, scene structures etc. depending on their own needs.

People who work on 3D processing usually know the math and how to construct a pipeline,
so they do not have to be taught from scratch.
As an analogy, in medicine it is useful to have a first-aid kit and learn how to use it.
A surgeon has a very different need for tools,
that can seem complex and overwhelming for people who do not understand the purposes of the tools.
3D processing is more like surgery than first-aid.

In 3D processing, it is rare to use e.g. exotic math.
The same math tools are used over and over, which does not require many dependencies.
What makes 3D processing challenging is in composition.
Here, people often over-engineer solutions, trying to cover every single purpose.
However, what professionals often neeed is a way to get started quickly,
but without sacrificing flexibility.
They need a way to quickly fix stuff and make precise changes with predictable results.

Think about a content production pipeline as a patient in homeostasis,
where the people who do 3D processing are doctors and surgeons.
They try to keep the patient alive and changes have to happen in harmony with the flow.
If they can not reason about the problem, then the patient has higher risk of dying (disrupting homeostasis).
Anything that gets in the way of this maintenance is counter-productive.

Reasoning about the problem is the main bottleneck in 3D processing.
It is not usually hardware, raw performance or knowledge about e.g. math.
These things are often supportive in the work, but they do not constitute the major part of the work.

This is the design philosophy of Turbine-Process3D.

