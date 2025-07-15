---
slug: running-kubernetes-with-raspberry-pi-5-nvme
title: Running Kubernetes with Raspberry Pi 5 and NVMe
description: The Raspberry Pi 5 represents a significant leap forward in the world of single-board computers, with its quad-core CPU, up to 8GB of RAM, and PCIe 2.0 interface making it an excellent candidate for running Kubernetes. When paired with high-speed NVMe storage instead of traditional SD cards, you can create a surprisingly powerful and reliable Kubernetes cluster suitable for development, testing, or even light production workloads. In this guide, we'll walk through the process of setting up K3s (a lightweight Kubernetes distribution) on a Raspberry Pi 5 with NVMe storage, configure secure access, and deploy essential packages to create a fully-functional Kubernetes environment.
preview: The Raspberry Pi 5 represents a significant leap forward in the world of single-board computers, with its quad-core CPU, up to 8GB of RAM, and PCIe 2.0 interface making it an excellent candidate for running Kubernetes. When paired with high-speed NVMe storage instead of traditional SD cards, you can create a surprisingly powerful and reliable Kubernetes cluster suitable for development, testing, or even light production workloads. In this guide, we'll walk through the process of setting up K3s (a lightweight Kubernetes distribution) on a Raspberry Pi 5 with NVMe storage, configure secure access, and deploy essential packages to create a fully-functional Kubernetes environment.
published_at: 2025-04-16T00:00:00+00:00
revised_at: 2025-07-15T00:00:00+00:00
---

For some time, I have run my ["homelab" on Google GKE](https://github.com/corybuecker/terraform-k8s-gke/tree/main). The free control plane, coupled with spot instances, meant a monthly bill around $50. With the release of the [16GB Raspberry Pi 5](https://www.raspberrypi.com/products/raspberry-pi-5/) with [NVMe support](https://www.raspberrypi.com/products/ssd-kit/), I have reduced the monthly $50 bill to around $5. Of course, the one-time hardware purchase is around $200 per node.

I'll walk through the process of setting up the Raspberry Pi with NVMe storage and K3s (a lightweight Kubernetes distribution), then deploy a few essential applications to create a fully-functional Kubernetes homelab environment.

## Special note

I ran into power delivery issues when using a standard phone charger with the Pi. I strongly recommend buying a high-quality 5.1V/5A (45W) power adapter. If you encounter random shutdowns during operation, insufficient power is most likely the cause. In the Syslog journal (`sudo journalctl -r`), I noticed specific errors related to undervolting.

## Prerequisites

My hardware list:

- Raspberry Pi 5 (the 16GB model)
- MicroSD card for initial setup
- NVMe SSD (I have the 256GB)
- Compatible NVMe HAT for Raspberry Pi 5
- Proper cooling solution (I have the $5 active cooler)

## Setting Up Raspberry Pi 5

Follow the official instructions for installing the [cooler](https://datasheets.raspberrypi.com/cooling/raspberry-pi-active-cooler-product-brief.pdf) and [M.2 HAT](https://datasheets.raspberrypi.com/ssd/raspberry-pi-ssd-kit-product-brief.pdf) on the Pi.

1. Install Raspberry Pi OS on your NVMe drive:
   - Follow the [official documentation](https://www.raspberrypi.com/documentation/computers/getting-started.html#installing-the-operating-system) for booting the Pi from an SD card
   - Make sure to use the 64-bit Desktop version of Raspberry Pi OS
2. After booting the Pi from the SD card, install the lite, 64-bit OS to the NVMe storage using the Raspberry Pi Imager tool. Be sure to set up network configuration and enable SSH access as there will not be a GUI after rebooting to the NVMe drive.
3. Shutdown the Pi, remove the SD card, and then start the Pi. If everything works, you should be able to SSH into the Pi after 20-30 seconds.
4. Update the Pi.

```bash
# Upgrade all packages

sudo apt update
sudo apt full-upgrade
```

## Pre-Kubernetes setup

1. Kubernetes requires cgroup support, so edit `/boot/firmware/cmdline.txt` and add `cgroup_memory=1 cgroup_enable=memory` to the end of the first line.

2. This step is optional but I disable the swap on the Pi.

```bash
# Disable swap
sudo dphys-swapfile swapoff

# Disable swap service
sudo systemctl disable dphys-swapfile

# Remove swap file
sudo dphys-swapfile uninstall

# Reboot
sudo reboot
```

## Installing K3s Without Traefik

K3s is a lightweight Kubernetes distribution perfect for resource-constrained environments like the Raspberry Pi. By default, it comes with Traefik (an ingress controller that manages external access to services) pre-installed, but I disabled it during installation so I could later configure a custom version with specific options.

1. Install K3s without Traefik:

```bash
curl -sfL https://get.k3s.io | sh -s - --disable=traefik
```

2. Verify your installation:

```bash
# Check the status of K3s
sudo systemctl status k3s

# Verify that the node is ready
sudo kubectl get nodes

# Check that all system pods are running
sudo kubectl get pods -A
```

You should see your Raspberry Pi listed as a node with status "Ready" and all system pods in the "Running" state.

## Setting Up Your Local Environment with Cluster Certificate

Now that K3s is running, set up the local environment to connect to that cluster.

1. Start a port-forwarding SSH session to the Raspberry Pi.

```bash
ssh -fN -L 6443:localhost:6443 <PI_IP_ADDRESS>
```

2. Copy the `kubeconfig` from your Raspberry Pi to your local machine:

```bash
# On the Raspberry Pi

sudo cat /etc/rancher/k3s/k3s.yaml

# On the local machine

mkdir -p ~/.kube
vim ~/.kube/config
```

Copy and paste the Pi's config into the local config file.

3. Verify that the connection from the local environment to the cluster.

```bash
kubectl get nodes
kubectl get pods -A
```

## Post-installation

At this point, you should have a fully functional K8s node. I have some extra packages in [raspberrypi-k8s](https://github.com/corybuecker/raspberrypi-k8s) that add some useful features such as Traefik, Prometheus, etc.

1. Clone the repository to your local machine:

```bash
git clone https://github.com/corybuecker/raspberrypi-k8s.git
cd raspberrypi-k8s
```

2. Install Traefik:

You don't need an ingress controller unless you want to route external network traffic to services running on the Pi. In a future post, I'll explain how I serve public websites, including this blog, with Traefik, a GCE proxy node, and [Tailscale](https://tailscale.com).

```bash
cd traefik

# Install Helm repo
helm repo add traefik https://traefik.github.io/charts
helm repo update

# Install Traefik
helm upgrade --namespace kube-system \
  --install -f routing.yaml traefik traefik/traefik
```

3. Install monitoring tools:

```bash
cd prometheus

# Apply Prometheus and Grafana from the repository
kubectl apply -n prometheus -k .
```

Running Kubernetes on a Raspberry Pi 5 with NVMe storage creates a surprisingly capable and cost-effective platform for containerized applications. In particular, the NVMe support significantly improves I/O performance compared to SD cards, creating a genuinely practical environment for real-world applications. While the initial hardware investment is around $200 per node, the ongoing costs are minimal compared to cloud alternatives.
