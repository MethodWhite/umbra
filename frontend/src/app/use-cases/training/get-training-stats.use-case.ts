import { Injectable } from '@angular/core';
import { TrainingRepository } from '../../data/repositories/training.repository';
import { TrainingStatus } from '../../domain/models/training.model';
import { UseCase } from '../../domain/interfaces/use-case.interface';

@Injectable({ providedIn: 'root' })
export class GetTrainingStatsUseCase implements UseCase<void, TrainingStatus | null> {
  constructor(private trainingRepo: TrainingRepository) {}

  async execute(): Promise<TrainingStatus | null> {
    return this.trainingRepo.getStatus();
  }
}
