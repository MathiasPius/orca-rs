# Orca

Orca is a meta-tool for building Packer iamges and deploying these to libvirt Hosts.

## Overview
At a high level, Orca defines a number of specs which outline:

1. How to build an image using Packer. What are the inputs (variables, dependent images, files) and outputs (qcow2 images)
2. How is a deployment of one or more images defined?



## Specifications

### Images
An Image specification defines the path of a Packer build, as well as any variables which will have to be passed into this build to produce an output. Any qcow2 images produced by this build are considered the outputs.

Images can also specify dependencies in the form of other images. Ideally these would be derived from the Packer build itself, but short of parsing the hcl, this is not feasible.

### Service
A Service specification details which images are to be deployed, and in which configuration.


## Technical Details
Orca assumes that .orca/cache is used as a scratch pad for building these images.